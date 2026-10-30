# Dataset Assistant

A GUI program for the creation and management of datasets.

Rapidly build a dataset for training. Groups images by position and intensity
and sweeps through camera controls.

## Project Information

The project uses eframe/egui as a framework to build a simple GUI application
for collecting image data into a dataset. The image data is tagged with the
control values used to capture it. The images from one invocation of the saving
procedure are bundled into a subset, which each subset being a unique
combination of a position and intensity; the subsets together compose the
dataset. The root dataset folder contains a `index.csv` file, that is used to
easily interact with it programatically. After selecting a camera, you are able
to configure the camera controls and define the camera control used for the
sweeps using during the saving procedure. The sweeps will run through every
permutation of the camera controls, as configured. After all subsets have been
built, a CSV builder can be executed to read the current dataset structure and
build the `index.csv` file.

## System Design

The [`application.rs`](src/application.rs) file houses the
`DatasetAssistantApp`, which orchestrates the high-level control flow and GUI
element placement. It operates very closely with
[`user_interface.rs`](src/user_interface.rs) and [`state.rs`](src/state.rs).
[`user_interface.rs`](src/user_interface.rs) was originally just slated to be a
module of related stateless UI functions, however this design choice has
resulted in a lot of painful coupling/external reliance as the state must be
managed elsewhere. This, of course, falls onto [`state.rs`](src/state.rs),
which holds the `State` type, an incredibly bloated struct that handles almost
the entire program state by itself. Work is underway to refactor this structure; `State` has been labelled with information helpful for decomposition and a `UIComponents` structure has been made to deal with UI state.

Since the main thread is managed by `eframe`, which sleeps when no user input is detected, any active processes must be run on a separate thread. The [`runner.rs`](src/runner.rs) file provides a handle to be able to track the progress and update the UI state accordingly.

The saving procedure is defined under [`saving.rs`](src/saving.rs) and runs on
a separate thread. It requires exclusive access to the camera controls to be
able to perform its duty. To manage all information about the camera controls,
the [`camera_controls.rs`](src/camera_controls.rs) file provides the
`ControlManager` type. During saving, the controls are iterated through by the
`ControlSweep` type provided by [`sweep.rs`](src/sweep.rs).

The CSV building logic is defined under [`csv_builder.rs`](src/csv_builder.rs)
and runs on a separate thread. It provides two different formats: standard and
legacy.

The filepaths are managed by a combination of [`dataset.rs`](src/dataset.rs) and [`subset.rs`](src/subset.rs).

## Submodules

The repository has the [`edge`](edge) repository as a submodule to get access
to the [`edge-cameras`](edge/crates/edge-cameras) crate. This was done to
mutually benefit both packages. [`edge-cameras`](edge/crates/edge-cameras)
provides an clean (if incomplete) interface over
[`nokhwa`](edge/submodules/nokhwa) to this project, and development here helps
inform design decisions and catch bugs for
[`edge-cameras`](edge/crates/edge-cameras).

## For Development

To enable the pre-commit hook, run `git config core.hooksPath .githooks`.
