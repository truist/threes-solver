# threes_simulator

An implementation of the [Threes!](https://en.wikipedia.org/wiki/Threes) game algorithm, as best as I could figure it out.

This crate was designed to be used as a library for use by threes_solver, but it also includes a `main()` that lets you play *Threes!* from within a shell!

To play:

`cargo run`

Press `q` to quit.


## Credits

This thread was hugely useful to help me understand the game algorithm: https://toucharcade.com/community/threads/threes-by-sirvo-llc.218248/page-27.
(I summarized the algorithm for myself in [`todo.txt`](../todo.txt).)


Other people have come before me:

* https://www.reddit.com/r/compsci/comments/33xmtt/comment/cqph6tp/
* http://blog.waltdestler.com/2014/04/threesus.html
    * https://github.com/waltdestler/Threesus?tab=readme-ov-file
* https://github.com/nneonneo/threes-ai


## License

[GPL v3.0](LICENSE-GPL3)

