# bevy_2d_inverse_kinematics

a [bevy](https://bevyengine.org/) plugin for 2d [inverse kinematics](https://en.wikipedia.org/wiki/Inverse_kinematics) solved by [FABRIK](http://www.andreasaristidou.com/FABRIK.html)


2d meaning it has 3 degrees of freedom (XY translation, Z rotation)

## FABRIK
Forward And Backward Reaching Inverse Kinematics

description [here](http://www.andreasaristidou.com/FABRIK.html)

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
![](https://github.com/ntibi/bevy_2d_inverse_kinematics/blob/master/misc/arm.gif)
![](https://github.com/ntibi/bevy_2d_inverse_kinematics/blob/master/misc/frog.gif)
