Hello everybody out there using Linux -

I'm doing a (free) operating system (just a hobby, won't be big and professional like Linux) for x86_64 CPUs. This has been brewing since late january, and is starting to get ready. I'd like any feedback on things not in Linux people like/dislike, as my OS doesn't resembles it somewhat.

I've currently rendered some graphics, and things seem to work. This implies that I'll get something practical within a few months, and I'd like to know what features most people would want. Any suggestions are welcome, but I won't promise I'll implement them :-)

It uses the rust-osdev's experimental x86_64 bootloader that works on both BIOS and UEFI systems, so no support for non x86_64 CPUs (unless I made another bootloader myself). I also don't aspire to support POSIX standards, but I might create a simple WASM runtime (if I'm interested) or any special and simple binary code format.

zefr0x (https://github.com/zefr0x)

PS. Yes - it's free of any Linux code, and it has no support for anything yet. It is NOT general purpose, and it probably never will has anything other than simple fs and basic graphics with some simple tools or a game, as that's all I'm interested in :-(.
