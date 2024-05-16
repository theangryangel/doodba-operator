# doodba-operator

:warning: A WIP operator to help run [Doodba](https://github.com/Tecnativa/doodba) based Odoo installations on Kubernetes.

**Right now this is 100% non-functional**

Managing Odoo through helm works absolutely fine... until you introduce a high
throughput system, or the job queue.

At this point standard helm functionality is started to be stretched: you need
to be able to do things like scale down deployments, wait for upgrades to
complete, etc.

You *can* implement init containers, upgrade hooks, etc. but it seems like an operator
becomes a better fit the more complex the needs of the installation.

This is a WIP operator that I am working on in my own personal time as an
experiment.

