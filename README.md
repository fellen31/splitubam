# splitbam
Hardly anyone wants to split one ubam into multiple, fast

# Usage
```
splitbam [OPTIONS] --split <SPLIT> <INPUT>

Arguments:
  <INPUT>  bam file to split

Options:
  -t, --threads <THREADS>  Number of parallel decompression & writer threads to use [default: 4]
  -s, --split <SPLIT>      Number of files to split bam to
  -h, --help               Print help
  -V, --version            Print version
```
# Example
```
splitbam test.bam --split 4 test.bam
```
creates 4 files:
```
001.test.bam  002.test.bam  003.test.bam  004.test.bam
```
