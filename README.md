# splitbam
Hardly anyone wants to split one ubam into multiple, per line, fast

# Usage
```
splitubam [OPTIONS] --split <SPLIT> <INPUT>

Arguments:
  <INPUT>  bam file to split

Options:
  -t, --threads <THREADS>          Number of parallel decompression & writer threads to use [default: 4]
  -s, --split <SPLIT>              Number of files to split bam to
  -c, --compression <COMPRESSION>  BAM output compression level [default: 6]
  -h, --help                       Print help
  -V, --version                    Print version


```
# Example
```
splitubam test.bam --split 4 test.bam
```
creates 4 files:
```
001.test.bam  002.test.bam  003.test.bam  004.test.bam
```
