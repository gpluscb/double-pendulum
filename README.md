# Double Pendulum Rendering

50,000 double pendulums.

https://github.com/gpluscb/double-pendulum/assets/20143778/b2389fde-aeb7-4e6a-bafa-d542312ec9ff

(if you can't play the video I blame your browser btw I know I can't and I blame firefox (vlc works))

Please keep in mind all of this is hacked together to get quick results one and a half years ago, and wasn't meant to be open sourced.
So code bad, sorrryyyy!

## How 2 Use

As it is, the code actually renders only 5,000 double pendulums, but that's an easy change in main.rs line 33.
So we have two modes of rendering, rendering to a window with sdl2 and rendering to pngs (I ended up stitching those to a vid with ffmpeg somehow I forgot how).
Flip between the two by setting the `render_in_window` bool accordingly in main.
If you render to window and you're cool you close the window by pressing the escape key that would be kind to the program I think.
Render to png I think ctrl-c works it doesn't care.
Changing initial config in code also should be straight forward I think I hope.
Make sure you make an `out` folder in the working directory, pngs will be saved there, and the final configuration of the last run will also be stored there in json.
Don't think there's a built-in way of reading it tho and continuing where it left off, but you'll figure it out somehow probably.

The current version for the image renderer renders transparent polygons between adjacent pendulums, but yea feel free to modify.

All the cool pendulum maths is in `core/mod.rs`, I still have the source for that math which I'm happy about because it's cool -> http://www.maths.surrey.ac.uk/explore/michaelspages/documentation/Double.pdf

Thank you Michael Hart!

# License

Uhhhh idk do whatever!! One rule if you make something cool or pretty using this I wanna see, @gpluscb on Twitter or MarRue#3658 on Discord
