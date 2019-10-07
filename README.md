# Stentorian
Stentorian is a safe Rust library for interacting with the speech recognition engine of Dragon NaturallySpeaking. It allows you to load speech recognition grammars into Dragon and receive notifications when a command has been spoken that matches the grammar. This library is mostly not intended to be used directly. Instead, you can use `stentorian-server`, which provides a high-level JSON-RPC API on top of this library. You can then connect to it using the Python library `stentorian-client`, which provides a very convenient way of linking up grammars to actual actions to be executed.

Since Dragon speech recognition grammars are based on regular expressions (on words, instead of characters), this library contains a custom regular expression engine, which produces parse trees. This is different from most other regular expression engines, which operate on characters, and which do not produce parse trees but only a sequence of "match groups". The use of parse trees makes it very easy to build up more complicated commands out of simpler commands. In the Python library, this is used for defining re-usable grammar components.

For instance, one can define a grammar
```
<number> := zero | one | two | three | ... | ten
```
to recognize numbers. Then, based on this, one might like to define a command for deleting a number of lines.
```
<delete_command> := (delete <number> lines)+
```
Note how we use `+` here to allow for multiple repetitions of the command. This would produce a parse tree that has a number of children matching the number of times `delete_command` has been spoken, and where each child node contains the number that has been spoken.
