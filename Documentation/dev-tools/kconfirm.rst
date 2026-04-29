.. SPDX-License-Identifier: GPL-2.0-only
.. Copyright (C) 2026 Julian Braha <julianbraha@gmail.com>

========
kconfirm
========

kconfirm is a static analysis tool for the kernel's Kconfig system.  It
checks the entire tree-wide Kconfig, and reports misusage like
dead code.  In the case of dead default statements, these can be a
significant code smell.

kconfirm has an optional check for dead links in the Kconfig help texts.
Since this has a high potential for false positives (due to websites
blocking bots) and slows down runtime signficantly, it is disabled by
default.  However, an example of how to enable it is included below.

kconfirm is written in Rust and lives in ``scripts/kconfirm``.  Other
than the dead link checks, kconfirm aims for zero false positives.

**NOTE**: kconfirm does not modify or compile the source tree; it is
strictly a static checker.


Getting Started
===============


kconfirm's Minimum Supported Rust Version (MSRV) is v1.85.0, because
it uses Rust edition 2024, and this is the earliest supported version.

kconfirm also requires the Cargo package manager and an internet
connection for compilation of its dependencies.

If Cargo is available, kconfirm can be built and run from the top of the
kernel source tree::

    make kconfirm

The compiled ``kconfirm-linux`` binary will be available in
``scripts/kconfirm/target/release/``.

The default checks currently cover dead code analysis.  ``dead_links``
must be turned on explicitly with ``--enable``; conversely, any default
check can be turned off with ``--disable``.  Both options accept
either a comma-separated list or repeated flags, so the following
two invocations are equivalent::

  kconfirm-linux --linux-path . --enable dead_defaults,dead_links
  kconfirm-linux --linux-path . --enable dead_defaults --enable dead_links



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

``--enable CHECK[,CHECK...]``

    Enable one or more checks in addition to the default set.  May be
    given multiple times, or as a single comma-separated list.  See
    `Available checks`_ below for valid names.

``--disable CHECK[,CHECK...]``

    Disable one or more checks from the default set.  May be given
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

``duplicate_select`` *(default)*

    Reports duplicated ``select`` entries on a single Kconfig symbol.

``duplicate_default`` *(default)*

    Reports duplicated ``default`` entries on a single Kconfig symbol.

``dead_default`` *(default)*

    Reports ``default`` entries that can never be selected, for example
    because their condition is unsatisfiable.

``dead_links``

    Reports broken URLs found in Kconfig help text.  Because this
    performs network requests it can be quite slow, and is disabled by
    default. May also have false positives.

``style``

    Reports opinionated style issues in Kconfig files.  Disabled by
    default.


Examples
========

Compile (as needed) and run on the current tree::

    make kconfirm

To additionally enable dead-link checking::

    make kconfirm KCONFIRM_ARGS="--enable dead_links"

To disable a check (here, ``duplicate_dependency``) while keeping the
rest of the default set::

    make kconfirm KCONFIRM_ARGS="--disable duplicate_dependency"

To run the default checks against a kernel tree separate from the
current directory, such as ``~/repos/linux``::

    scripts/kconfirm/target/release/kconfirm-linux --linux-path ~/repos/linux
