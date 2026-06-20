# The Kubernetes cluster + the Kata Containers runtime install.
#
# Cluster creation itself is cloud-specific, so it is left as a documented choice
# (uncomment the module for your provider). What IS cloud-neutral and IS wired up
# here is installing the Kata runtime onto the cluster's nodes via kata-deploy -
# that is the part that turns a generic cluster into one that can run RustAG's
# isolated tenant pods.

# --- Option A: managed cluster (recommended) --------------------------------
# Pick one. These community modules create the cluster and a kubeconfig you then
# point versions.tf's providers at.
#
# module "eks" {
#   source          = "terraform-aws-modules/eks/aws"
#   version         = "~> 20.0"
#   cluster_name    = var.cluster_name
#   cluster_version = "1.30"
#   # Kata requires bare-metal or nested-virtualization-capable instance types.
#   eks_managed_node_groups = {
#     kata = { instance_types = ["m5.metal"], min_size = var.node_count, max_size = var.node_count + 2 }
#   }
# }
#
# module "gke" {
#   source     = "terraform-google-modules/kubernetes-engine/google"
#   project_id = var.gcp_project
#   name       = var.cluster_name
#   # GKE Sandbox (gVisor) or a node pool with nested virt for Kata/Firecracker.
# }

# --- Kata runtime install (cloud-neutral) -----------------------------------
# kata-deploy runs a DaemonSet that installs the Kata binaries + containerd
# config onto each labelled node, then we register the RuntimeClass
# (see ../kubernetes/kata-runtimeclass.yaml, applied separately or below).
resource "helm_release" "kata_deploy" {
  name             = "kata-deploy"
  repository       = "https://kata-containers.github.io/kata-containers-charts"
  chart            = "kata-deploy"
  namespace        = "kube-system"
  create_namespace = false

  # Restrict the install to nodes you've labelled Kata-capable.
  set {
    name  = "k8sDistribution"
    value = "k3s" # change to "rke2"/"eks"/"gke" to match your cluster
  }
}

# Apply the RuntimeClass manifest once kata-deploy has placed the runtime.
resource "kubernetes_manifest" "kata_runtimeclass" {
  depends_on = [helm_release.kata_deploy]
  manifest = yamldecode(file("${path.module}/../kubernetes/kata-runtimeclass.yaml"))
}

# Namespace that holds tenant stagenet pods (separate from the control plane).
resource "kubernetes_namespace" "stagenets" {
  metadata {
    name = "rustag-stagenets"
    labels = {
      "pod-security.kubernetes.io/enforce" = "restricted"
    }
  }
}
