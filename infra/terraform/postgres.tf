# Postgres - the swap target for the MVP's SQLite control-plane store.
#
# Budget path (default): self-hosted on the cluster via the Bitnami chart, with a
# persistent volume. For production HA, replace this whole file with a managed
# instance (aws_db_instance / google_sql_database_instance) - the connection
# string is the only thing the app consumes, so the swap is contained here.
#
# The DDL the app runs against this is already Postgres-portable
# (see ../../migrations); TimescaleDB is enabled below so the `metrics` table can
# become a hypertable for the analytics time-series.

resource "kubernetes_namespace" "data" {
  metadata { name = "rustag-data" }
}

resource "helm_release" "postgres" {
  name       = "rustag-postgres"
  repository = "https://charts.bitnami.com/bitnami"
  chart      = "postgresql"
  version    = "~> 15.5"
  namespace  = kubernetes_namespace.data.metadata[0].name

  set {
    name  = "auth.username"
    value = "rustag"
  }
  set {
    name  = "auth.database"
    value = "rustag"
  }
  set_sensitive {
    name  = "auth.password"
    value = var.postgres_password
  }
  set {
    name  = "primary.persistence.size"
    value = "${var.postgres_storage_gb}Gi"
  }
  # TimescaleDB extension image for the analytics hypertable.
  set {
    name  = "image.repository"
    value = "timescale/timescaledb"
  }
  set {
    name  = "image.tag"
    value = "2.17.2-pg16"
  }
}

# The Secret the control plane and stagenet pods read DATABASE_URL from.
resource "kubernetes_secret" "db_url" {
  metadata {
    name      = "rustag-db"
    namespace = kubernetes_namespace.data.metadata[0].name
  }
  data = {
    url = "postgres://rustag:${var.postgres_password}@rustag-postgres-postgresql.${kubernetes_namespace.data.metadata[0].name}.svc:5432/rustag"
  }
  type = "Opaque"
}
