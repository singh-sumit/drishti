---
title: systemd Integration
sidebar_position: 3
---

The repository includes a hardened unit file in `deploy/drishti.service`.

## Install

```bash
sudo cp deploy/drishti.service /etc/systemd/system/drishti.service
sudo systemctl daemon-reload
sudo systemctl enable --now drishti.service
```

## Verify

```bash
systemctl status drishti.service
journalctl -u drishti.service -f
```

## Operational Notes

- ensure `metrics_addr` is reachable by Prometheus
- ensure configured `pid_file` location is writable by service user
- keep collector toggles conservative on low-resource nodes
