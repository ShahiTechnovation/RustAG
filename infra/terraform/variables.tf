# Inputs for the RustAG cloud infrastructure. Set these in a terraform.tfvars
# file (gitignored) or via TF_VAR_* env vars. Never commit secret values.

variable "cluster_name" {
  description = "Name of the Kubernetes cluster hosting RustAG cloud."
  type        = string
  default     = "rustag-cloud"
}

variable "region" {
  description = "Cloud region for the cluster and datastores."
  type        = string
  default     = "us-east-1"
}

variable "kubeconfig_path" {
  description = "Path to the kubeconfig for the target cluster."
  type        = string
  default     = "~/.kube/config"
}

variable "node_count" {
  description = "Number of worker nodes. Kata needs nested-virt-capable nodes."
  type        = number
  default     = 3
}

variable "base_domain" {
  description = "Apex domain; projects get <slug>.<base_domain>."
  type        = string
  default     = "stagesvm.dev"
}

variable "postgres_password" {
  description = "Password for the RustAG Postgres role. Pass via TF_VAR_postgres_password."
  type        = string
  sensitive   = true
}

variable "postgres_storage_gb" {
  description = "Persistent volume size for Postgres."
  type        = number
  default     = 20
}

variable "redis_password" {
  description = "Password for Redis. Pass via TF_VAR_redis_password."
  type        = string
  sensitive   = true
}

variable "acme_email" {
  description = "Contact email for Let's Encrypt (cert-manager ClusterIssuer)."
  type        = string
}
