# Panama

A cross-platform terminal emulator with (planned) built-in LLM support. Panama is currently in active development, and is being built with the following technologies: 

- Tauri for its lightweight, cross-platform native desktop application framework
- Rust for high-performance, multi-threaded low-level systems integration
- Typescript + SolidJS for a reactive, performant user interface

The vision for the project is to allow the user to input some "context" about what they are doing in the terminal - and then get some autocomplete suggestions from an LLM that are tailored to that context. 

For example - let's say the user has just SSH'ed into a new Linux environment and is looking to follow the typical environment setup process - installing/updating necessary applications, setting up users and a firewall, obtaining an SSL certificate, downloading and installing Docker, etc. By providing a plain text description of what they are doing - something like, "I am setting up this newly installed Ubuntu environment to run my Node application in Docker and expose it via port 80 with an SSL certificate," - then the LLM can provide autocomplete suggestions that align with the user's goals.

It might also be reasonable, as a stretch feature, to experiment with the idea of allowing the LLM to also pass ANSI escape sequences to move the cursor, helping the user edit files in applications like Nano or VIM. This might require a specially trained model, and a specialized interface to allow the user to preview the action in some way.

## Contributing

If you'd like to help, please feel free to open a new issue regarding any features or bugs. Feel free to fork, clone, install (with `yarn`), and run (with `yarn tauri dev`) the project yourself - just make sure you have Node and Rust installed.
