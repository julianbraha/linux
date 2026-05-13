.. SPDX-License-Identifier: GPL-2.0-only
.. Copyright (C) 2026 Julian Braha <julianbraha@gmail.com>

========
kconfirm
========

kconfirm is a static analysis tool for the kernel's Kconfig.  It checks
the entire tree-wide Kconfig, and reports misusage like dead code.  In the
case of dead default statements, these can be a code smell.

kconfirm has some additional, optional checks. The first is for dead links
in the Kconfig help texts.  Since this has a high potential for false
positives (due to websites blocking bots) and slows down runtime
significantly, it is disabled by default.

Another optional check is for config options that select visible config
options.  Examples of how to enable the optional checks are included
below.

kconfirm is written in Rust and lives in ``scripts/kconfirm``.  Other
than the dead link checks, kconfirm aims for zero false positives.

By default, kconfirm checks the same architecture as your kernel build,
but you can also enable checking more architectures with
``--enable-arch`` or disable checking your default architecture with
``--disable-arch``.  Alarms are deduplicated across all affected
architectures; kconfirm displays a tag with the corresponding Kconfig
architecture config option names.  For example, ``[RISCV]`` indicates
that an alarm is specific to RISC-V, while ``[ARM, X86]`` indicates that
an alarm affects both arm and x86. Running on each architecture will take
approximately one minute on modern consumer hardware.

**NOTE**: kconfirm does not modify or compile the source tree; it is
strictly a static checker.


Getting Started
===============


kconfirm's Minimum Supported Rust Version (MSRV) is v1.85.0, because
it uses Rust edition 2024, and this is the earliest supported version.

kconfirm requires the Cargo package manager and an internet connection
to download its dependencies from crates.io.

In ``scripts/kconfirm/`` run the following to download the dependencies::

  cargo vendor

Then, kconfirm can be built and run from the top of the
kernel source tree::

  make kconfirm

The compiled ``kconfirm-linux`` binary will be available in
``scripts/kconfirm/target/release/``.

The default checks currently cover dead code analysis, as well as invalid
(reverse) ranges and constant conditions.  ``select_visible`` and
``dead_links`` must be turned on explicitly with ``--enable-check``;
conversely, any default check can be turned off with ``--disable-check``.  Both
options accept either a comma-separated list or repeated flags, so the
following two invocations are equivalent::

  kconfirm-linux --linux-path . --enable-check select_visible,dead_link
  kconfirm-linux --linux-path . --enable-check select_visible --enable-check dead_link


Options
=======

**NOTE**: kconfirm's arguments must be provided in the ``KCONFIRM_ARGS``
environment variable if running with ``make``. See `Examples`_.

Available options:

``--linux-path PATH``
  The path to the linux source tree to analyze. ``make`` uses this
  option to pass the current linux tree, but this option can be used
  when running the tool directly with another source tree.
  See `Examples`_.

``--enable-check CHECK[,CHECK...]``

  Enable one or more checks in addition to the default set.  May be
  given multiple times, or as a single comma-separated list.  See
  `Available checks`_ below for valid names.

``--disable-check CHECK[,CHECK...]``

  Disable one or more checks from the default set.  May be given
  multiple times, or as a single comma-separated list.

``--enable-arch ARCH[,ARCH...]``

    Enable one or more architectures in addition to the default
    architecture.  May be given multiple times, or as a single
    comma-separated list.

``--disable-arch ARCH[,ARCH...]``

    Disable one or more architectures from the default set.  May be given
    multiple times, or as a single comma-separated list.

``-h, --help``

  Show the help message and exit.

``-V, --version``

  Show version information and exit.


Available checks
================

Each check has a string name that is accepted by ``--enable`` and
``--disable``.  Checks marked *(default)* are enabled unless turned off
explicitly.

``duplicate_dependency`` *(default)*

  Reports duplicated ``depends on`` entries on a single Kconfig symbol.

``duplicate_range`` *(default)*

  Reports duplicated ``range`` entries on a single Kconfig symbol.

``dead_range`` *(default)*

  Reports ``range`` entries that will never be evaluated, due to an
  unconditional range entry.

``duplicate_select`` *(default)*

  Reports duplicated ``select`` entries on a single Kconfig symbol.

``dead_select`` *(default)*

  Reports dead ``select`` entries that will never be evaluated, due to an
  unconditional select entry of the same config option.

``duplicate_imply`` *(default)*

  Reports duplicated ``imply`` entries on a single Kconfig symbol.

``dead_imply`` *(default)*

  Reports dead ``imply`` entries that will never be evaluated, due to an
  unconditional imply entry for the same config option.

``duplicate_default`` *(default)*

  Reports duplicated ``default`` entries on a single Kconfig symbol.

``dead_default`` *(default)*

  Reports ``default`` entries that can never be selected, for example
  because their condition is unsatisfiable.

``constant_condition`` *(default)*

  Reports conditions for any entries that always evaluate to ``true``.

``reverse_range`` *(default)*

  Reports invalid ranges for int and hex configuration options.

``failed_parse`` *(default)*

  Reports a parsing failure of the Kconfig. Cannot be disabled.

``select_visible``

  Reports configuration options that ``select`` a config option that is
  visible to users.

``dead_link``

  Reports broken URLs found in Kconfig help text.  Because this
  performs network requests it can be quite slow, and is disabled by
  default. May also have false positives.

``ungrouped_attribute``

  Reports ungrouped entries, like ``select`` and ``depends on``.
  This is a style check, and is disabled by default.

``duplicate_default_value``

  Reports duplicate default values that have different conditions.
  Suggests combining the conditions using a logical-or ``||``.
  This is a style check, and is disabled by default.


Examples
========

Compile (as needed) and run on the current tree::

  make kconfirm

To additionally enable the dead link and select-visible checks::

  make kconfirm KCONFIRM_ARGS="--enable-check=dead_link,select_visible"

To disable a check (here, ``duplicate_dependency``) while keeping the
rest of the default set::

  make kconfirm KCONFIRM_ARGS="--disable-check duplicate_dependency"

To enable an architecture (here, ``RISC-V``) while keeping the
default architecture enabled::

  make kconfirm KCONFIRM_ARGS="--enable-arch riscv"

To run the default checks against a kernel tree separate from the
current directory, such as ``~/repos/linux``::

  scripts/kconfirm/target/release/kconfirm-linux --linux-path ~/repos/linux
