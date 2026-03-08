# rng_util

A small wrapper for some Rng implementation details so they can be abstracted away from threes_simulator and threes_solver.
It makes it easy to bootstrap seeds from entropy and/or let users provide a saved seed, without having to hard-code a specific Rng implementation in the project.
(The implementation is hard-coded in this library.)

## License

This crate is licensed with the [MIT license](LICENSE-MIT).

