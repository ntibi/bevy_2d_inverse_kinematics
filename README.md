# FABRIK
Forward And Backward Reaching Inverse Kinematics

description [here](http://www.andreasaristidou.com/FABRIK.html)

## what's in the crate
a bevy plugin for 2d inverse kinematics solved by FABRIK

2d meaning it has 3 degrees of freedom (XY translation, Z rotation)

## examples
basic IK arm that follows the mouse cursor
```
cargo run --example arm
```

cute frog procedurally walking with IK animated leg
```
cargo run --example frog
```

equally cute frog from a gltf model with a single arm that follows the mouse
```
cargo run --example model
```
