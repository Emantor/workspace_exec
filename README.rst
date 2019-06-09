================
 workspace_exec
================

Rust implementation of https://github.com/i3/i3/issues/760, using a mapping file
with the default location :code:`~/.config/sway/workspace_exec/mapping.yaml`.

Example file:

.. code-block:: yaml

   mapping:
     myworkspace: ~/my/custom/location
     otherworkspace: ~/other/location

Usage in sway config file:

.. code-block:: cfg

   set $term workspace_exec -- termite
