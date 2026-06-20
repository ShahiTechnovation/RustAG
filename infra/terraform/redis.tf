# Redis - the swap target for the MVP's in-process (moka) cache.
#
# Required once the control plane runs as more than one replica: an in-process
# cache cannot be shared across replicas, so the shared account/blockhash cache
# and the slug→port routing table move to Redis. Self-hosted via the Bitnami
# chart here; swap for a managed instance (ElastiCache / Memorystore) by editing
# this file only.

resource "helm_release" "redis" {
  name       = "rustag-redis"
  repository = "https://charts.bitnami.com/bitnami"
  chart      = "redis"
  version    = "~> 20.1"
  namespace  = kubernetes_namespace.data.metadata[0].name

  # Single primary + replicas for read scaling; the app uses deadpool-redis with
  # a connection manager so failover is transparent.
  set {
    name  = "architecture"
    value = "replication"
  }
  set {
    name  = "replica.replicaCount"
    value = "2"
  }
  set_sensitive {
    name  = "auth.password"
    value = var.redis_password
  }
}

resource "kubernetes_secret" "redis_url" {
  metadata {
    name      = "rustag-redis"
    namespace = kubernetes_namespace.data.metadata[0].name
  }
  data = {
    url = "redis://:${var.redis_password}@rustag-redis-master.${kubernetes_namespace.data.metadata[0].name}.svc:6379"
  }
  type = "Opaque"
}
