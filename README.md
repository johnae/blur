## What's this?

It's what I use for my [sway](https://github.com/swaywm/sway) lockscreen. This uses an imperfect version of gaussian blurring which is MUCH faster than say the imagemagick one while the output is such that noone will notice any difference. The algorithm used here originally comes from http://blog.ivank.net/fastest-gaussian-blur.html.

This is very much a WIP and maybe not usable by anyone but me. Thank you to https://github.com/fschutt/fastblur for the original fast blur implementation in Rust.

The whole reason for this is that I wanted to do something like [swaylock-fancy](https://github.com/Big-B/swaylock-fancy) in less time since I find it awful to bind a key to locking your screen but you must wait 3+ seconds before you see anything happen - I just wanted sub second response time.

Here's how I use this on NixOS as my screenlocker: [nixos-configuration/swaylock-dope](https://github.com/johnae/nixos-configuration/blob/master/pkgs/swaylock-dope/swaylock-dope).