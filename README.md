# 2D Gravity Simulator

This project is me experimenting with Rust and Vulkan by developing
a gravity simulator of particles.

The gravity calculation for each particle is optimized i a way so
that we avoid n-squared complexity of the whole thing. It is done by
not calculating cumulative force among all particles but only by 
centers of masses of force regions of different region levels.

Before calculating force, it calculates mass fields in different
detail levels of images where for each pixel we store an accumulated
mass and a center of mass of that pixel. Then gravity is calculated
for each particle by evaluating a force that it should attract for all 
neighbouring regions of the particle in all detail levels.

To run the project:
`cargo run`
