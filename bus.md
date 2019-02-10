# MARIE Bus
Hook input and output instructions to a bus, allowing for greater interaction with your MARIE or Mars programs.

## Stream explanation
A MARIE program would issue commands using Output.

After commanding the bus to prepare a stream for reading, the MARIE program would issue Input instructions until it is satisfied.
The next Output instruction would be another command.

Writing to a stream would take a different approach.
After issuing the write command, all output instructions would write to the stream.
The next Input command would read a word, and finish the write, allowing the next Output instruction to issue a command for the bus.

## Command table
## (Draft)
```
Every command is one word, with 4 bits of instruction and 12 bits of arguments. (familar?)
0 Disable Bus (enter compatibility mode)
1 Read from stream
2 Write to stream
3 Open file as stream
    Arguments specify Options such as granularity (word, bit or byte)
    Write the filename, then INPUT to get the Stream ID.
    Directories work for reading here too
4
5
```

## STDIO
### (WIP)
Stream 0 is STD I/O
