
```
PLAY(1)                          User Commands                         PLAY(1)

NAME
       play - A simple command-line audio player.

SYNOPSIS
       play [OPTIONS] AUDIO_FILE

DESCRIPTION
       The play command allows you to play audio files directly from the
       command line. It supports common audio formats such as WAV, MP3, and
       FLAC.

ARGUMENTS
       AUDIO_FILE
              Path to the audio file you want to play.

OPTIONS
       -h, --help
              Display this help and exit.

       -V, --version
              Output version information and exit.

USAGE EXAMPLES
       Play an audio file:

           play song.wav

       Display help information:

           play --help

       Check the version of play:

           play --version

EXIT STATUS
       0      The program executed successfully.

       1      An error occurred (e.g., file not found, decoding error).

DEPENDENCIES
       This program requires the Rust `rodio` crate for audio playback and
       `clap` for command-line argument parsing.

       Ensure that your system has the necessary audio backends:

       - On Linux, you may need ALSA or PulseAudio.
       - On macOS, it uses the native CoreAudio.
       - On Windows, it uses WASAPI.

AUTHOR
       Your Name <your.email@example.com>

COPYRIGHT
       © 2024 cuervo-blanco. This program is distributed under the MIT License.

SEE ALSO
       aplay(1), mpg123(1), sox(1)

```
