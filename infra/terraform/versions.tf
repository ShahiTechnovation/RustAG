# Provider + version pins for the RustAG cloud infrastructure.
#
# Cloud-neutral by default: datastores and the Kata runtime install go through the
# `kubernetes` and `helm` providers, so this applies to any conformant cluster.
# To use a managed cluster/datastore instead, uncomment the relevant cloud
# provider here and edit kubernetes-cluster.tf / postgres.tf accordingly.

terraform {
  required_version = ">= 1.6"

  required_providers {
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.31"
    }
    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.15"
    }
    # aws = { source = "hashicorp/aws", version = "~> 5.0" }   # for EKS + RDS
    # google = { source = "hashicorp/google", version = "~> 6.0" } # for GKE + Cloud SQL
  }

  # Remote state so the team shares one source of truth. Point this at your bucket.
  # backend "s3" {
  #   bucket = "rustag-tf-state"
  #   key    = "cloud/terraform.tfstate"
  #   region = "us-east-1"
  # }
}

# These assume a kubeconfig already pointing at the target cluster (created in
# kubernetes-cluster.tf or out-of-band). For an in-Terraform-created cluster,
# wire these to the cluster module's outputs instead.
provider "kubernetes" {
  config_path = var.kubeconfig_path
}

provider "helm" {
  kubernetes {
    config_path = var.kubeconfig_path
  }
}
