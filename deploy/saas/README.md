# SaaS 部署(占位)

骨架阶段不交付 SaaS 部署细节,仅预留目录。Phase 1 之后填写:

- `k8s/`         — Helm chart(values.yaml + templates/)
- `terraform/`   — 云资源(RDS / ElastiCache / EKS 等)
- `argocd/`      — GitOps manifests
- `ingress/`     — TLS 终止 + 反向代理(NGINN / Caddy / Traefik)
- `observability/` — Prometheus + Grafana + Loki 配置

参考:本地一键装走 `deploy/install.sh`(基于 `deploy/docker-compose.yml`),
SaaS 形态在此基础上替换 compose 为 k8s + 外部托管 DB/Cache,核心镜像相同
(`gpai/core:latest`、`gpai/gateway:latest`、`gpai/web:latest`)。