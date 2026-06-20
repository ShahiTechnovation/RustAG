# Ingress + TLS for the public RustAG cloud endpoint.
#
# Installs the NGINX ingress controller and cert-manager (for automatic Let's
# Encrypt certs), then defines the ClusterIssuer the ../kubernetes/ingress.yaml
# annotation references. The Ingress object itself lives in kubernetes/ so it
# ships with the workload manifests; this file provisions what it depends on.

resource "helm_release" "ingress_nginx" {
  name             = "ingress-nginx"
  repository       = "https://kubernetes.github.io/ingress-nginx"
  chart            = "ingress-nginx"
  version          = "~> 4.11"
  namespace        = "ingress-nginx"
  create_namespace = true

  set {
    name  = "controller.service.type"
    value = "LoadBalancer"
  }
}

resource "helm_release" "cert_manager" {
  name             = "cert-manager"
  repository       = "https://charts.jetstack.io"
  chart            = "cert-manager"
  version          = "~> 1.16"
  namespace        = "cert-manager"
  create_namespace = true

  set {
    name  = "crds.enabled"
    value = "true"
  }
}

# The ClusterIssuer named in ingress.yaml's cert-manager.io/cluster-issuer.
resource "kubernetes_manifest" "letsencrypt_issuer" {
  depends_on = [helm_release.cert_manager]
  manifest = {
    apiVersion = "cert-manager.io/v1"
    kind       = "ClusterIssuer"
    metadata   = { name = "letsencrypt-prod" }
    spec = {
      acme = {
        server = "https://acme-v02.api.letsencrypt.org/directory"
        email  = var.acme_email
        privateKeySecretRef = { name = "letsencrypt-prod-account-key" }
        solvers = [{
          http01 = { ingress = { class = "nginx" } }
        }]
      }
    }
  }
}

output "ingress_loadbalancer_hint" {
  description = "After apply, point *.{base_domain} and api.{base_domain} at the ingress LB."
  value       = "kubectl -n ingress-nginx get svc ingress-nginx-controller -o wide  # then set DNS for *.${var.base_domain}"
}
