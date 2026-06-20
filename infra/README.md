# infra/ - Infrastructure as Code for the RustAG cloud

This directory is the **production hosting layer** for RustAG cloud. It is the
swap target for the MVP substitutions documented in
[`../docs/known-gaps.md`](../docs/known-gaps.md): where the single-node control
plane uses child-process isolation and SQLite today, these manifests describe the
multi-node, Kata-isolated, Postgres-backed deployment.

> Status: **scaffolding**. These files are valid, reviewed, and ready to apply,
> but a live cluster is not stood up in this repo. Applying them provisions real,
> billable infrastructure - read each file and set the variables first.

```
infra/
├── terraform/        # cluster + datastore provisioning (HCL)
│   ├── versions.tf
│   ├── variables.tf
│   ├── kubernetes-cluster.tf   # the K8s cluster (managed control plane)
│   ├── postgres.tf             # Postgres (the moka/SQLite swap target)
│   ├── redis.tf                # Redis (the in-process-cache swap target)
│   └── networking.tf           # ingress controller + cert-manager
├── kubernetes/       # workload manifests (apply after the cluster exists)
│   ├── kata-runtimeclass.yaml      # the security boundary: RuntimeClass "kata"
│   ├── stagenet-pod-template.yaml  # one hosted stagenet, Kata-isolated
│   └── ingress.yaml                # per-subdomain routing to the control plane
└── grafana/          # observability (the tracing→OTLP swap target)
    ├── dashboards/   # importable dashboard JSON
    └── alerts/       # Prometheus/Grafana alert rules
```

## Why this shape

- **Kata Containers (Firecracker backend) for tenant isolation.** RustAG stagenets
  run untrusted Solana program bytecode. A shared-kernel container escape would
  expose every co-tenant; a microVM keeps each tenant behind a hypervisor
  boundary. `kata-runtimeclass.yaml` is that boundary; `stagenet-pod-template.yaml`
  opts each pod into it with `runtimeClassName: kata`. See
  [`../docs/threat-model.md`](../docs/threat-model.md).
- **Postgres + Redis** replace SQLite + moka once there are multiple control-plane
  replicas writing concurrently. The DDL in [`../migrations`](../migrations) is
  already Postgres-portable.
- **Cloud-neutral Terraform.** The datastore and ingress resources use the
  `kubernetes` + `helm` providers (self-hosted on the cluster - the budget option
  the Phase 2 spec calls out), so this does not lock to a single cloud vendor.
  Swap in a managed `aws_db_instance` / `google_sql_database_instance` by editing
  `postgres.tf` only.

## Apply order

```bash
cd infra/terraform
terraform init && terraform apply            # 1. cluster + datastores

# 2. install the Kata runtime on the nodes (operator), then:
kubectl apply -f ../kubernetes/kata-runtimeclass.yaml
kubectl apply -f ../kubernetes/ingress.yaml

# 3. the control plane creates stagenet pods from the template at runtime
#    (kube-rs orchestrator - see docs/known-gaps.md "swap path").
```

## What is NOT here (intentionally)

Secrets. `DATABASE_URL`, the Clerk/Stripe keys, and upstream RPC tokens are
injected as Kubernetes Secrets created out-of-band (or via a sealed-secrets /
external-secrets operator), never committed. The manifests reference them by name
(`secretKeyRef`), never by value.
