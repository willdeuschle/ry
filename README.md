# ry
ry searches yaml for matching paths/values.

It's written in rust and inspired by [yq](https://github.com/mikefarah/yq). See [**Benchmarking**](https://github.com/willdeuschle/ry#benchmarking) for some simple comparisons between the two.

---

## Usage

`ry <yaml_file_name> <path_expression>`

Returns the nodes in `yaml_file_name` matching `path_expression`. See [**Basic**](https://github.com/willdeuschle/ry#basic) for `path_expression` examples.

## Basic

For a file `test_map.yml` containing:
```
a:
  b: 1
```
then:
```
ry test_map.yml 'a.b'
```
will return `1`.

And for a file `test_array.yml` containing:
```
letters:
  - a
  - b
```
then:
```
ry test_array.yml 'letters[1]'
```
will return `b`. 

---

### From STDIN

For the `test_map.yml` file above, the following:
```
cat test.yml | target/release/ry - 'a.b'
```
will return `1`. Note that the `-` character represents `STDIN`.

### Wildcard matching

The `'*'` character acts as a wildcard in path expressions. For a file `test_wild.yml` containing:
```
a:
  item_b:
    f: 1
  thing_c:
    f: 2
  item_d:
    f: 3
```
then:
```
ry test_wild.yml 'a.*.f'
```
will return:
```
1
2
3
```

You can also use wildcards with a prefix, for example:
```
ry test_wild.yml 'a.item*.f'
```
will return:
```
1
3
```

Finally, wildcards can be used to match array elements as well. For the file `test_wild_array.yml` containing:
```
letters:
  - a
  - b
  - c
```
then:
```
ry test_wild_array.yml 'letters[*]'
```
will return:
```
a
b
c
```

### Deep splat matching

The deep splat pattern `'**'` is used to recursively match nodes in a file. For the file `test_splat.yml` containing:
```
a:
  b1:
    c: # MATCH
      c: crabs_1 # MATCH
    d: bears
  b2:
    c: crabs_2
    d: bears
  b3:
    e:
      - f:
          c: crabs_3 # MATCH
          g: bears
      - f:
          g:
            c: crabs_4 # MATCH
            h: bears
      - f: bears
```
then:
```
ry test_splat.yml 'a.**.c'
```
will return:
```
crabs_1
c: crabs_1
crabs_2
crabs_3
crabs_4
```

### Printing matching paths

The `--printMode` feature flag allows you to print the paths leading to matching nodes. For a file `test_wild.yml` containing:
```
a:
  item_b:
    f: 1
  thing_c:
    f: 2
  item_d:
    f: 3
```
then:
```
ry test_wild.yml 'a.*.f' --printMode p
```
will return:
```
a.item_b.f
a.thing_c.f
a.item_d.f
```
while:
```
ry test_wild.yml 'a.*.f' --printMode pv
```
will return both matching paths and values:
```
a.item_b.f: 1
a.thing_c.f: 2
a.item_d.f: 3
```
Note that:
```
ry test_wild.yml a.*.f --printMode v
```
is equivalent to the default:
```
ry test_wild.yml 'a.*.f'
```
which will just return matching values:
```
1
2
3
```

### Multi-doc support
If you have multiple documents in a single yaml file, the `-d` feature flag allows you to apply your search to a specific document. By default all documents are searched.

For the file `test_multi_doc.yml` containing:
```
a:
  b: 1
crabs: here
---
a:
  b: 2
```
then:
```
ry test_multi_doc.yml 'a.b' -d1
```
will return `2`, while:
```
ry test_multi_doc.yml 'a.b' -d0
```
will return `1`. Finally:
```
ry test_multi_doc.yml 'a.b' -d'*'
```
will return:
```
1
2
```

### Collecting results into an array
The feature flag `--collect` will collect the output elements into an array. For the file `test_collect.yml` containing:
```
letters:
  a:
    crab: 1
  b:
    crab: 2
  c:
    crab: 3
```
then:
```
ry test_collect.yml 'letters.*.crab'
```
will return:
```
1
2
3
```
while:
```
ry test_collect.yml 'letters.*.crab' --collect
```
will return:
```
- 1
- 2
- 3
```

### Printing the length of results
The `--length` feature flag prints the length of results.

For arrays, length means the number of items. For the file `test_array_length.yml` containing:
```
looking:
  - here
  - there
  - elsewhere
```
then:
```
ry test_array_length.yml 'looking' --length
```
will return `3`.

For maps, length means the number of entries. For the file `test_map_length.yml` containing:
```
looking:
  here: yes
  there: yes
  elsewhere: no
```
then:
```
ry test_map_length.yml 'looking' --length
```
will return `3`.

Finally for scalars, length means the length of the scalar. For the file `test_string_length.yml` containing:
```
looking: string
```
then:
```
ry test_string_length.yml 'looking' --length
```
will return `6`. For the file `test_int_length.yml` containing:
```
looking: 100
```
then:
```
ry test_int_length.yml 'looking' --length
```
will return `3`.

### Anchors and Aliases
Anchors and aliases will be substituted automatically. This means that for a file `anchor_and_alias.yml` containing:
```
first: &crab
  a: b
second: *crab
```
then:
```
ry anchor_and_alias.yml 'second.a'
```
will return `b`.

## Advanced

### Filtering by children nodes
You can filter parent nodes based on their children. For the file `test_filter.yml` containing:
```
a:
  - b:
      c: magic
    d: crab
  - b:
      c: magically
    d: bear
  - b:
      c: magic
    d: more crab
  - b:
      careful: magic
    d: most crab
```
then
```
ry test_filter.yml 'a.(b.c==magic).d' --printMode pv
```
will return:
```
a[0].d: crab
a[2].d: more crab
```
> Note that `(b.c==magic)` filters based on the children of each array member under `a`, but it returns the parents of those children for further searching (i.e. the filter `(b.c==magic)` returns array members under `a`).

Filtering also supports wildcards, for example:
```
ry test_filter.yml 'a.(b.c*==magic*).d' --printMode pv
```
will return:
```
a[0].d: crab
a[1].d: bear
a[2].d: more crab
a[3].d: most crab
```

### Matching on children values
Similar filtering by children nodes, it's also possible to filter based on children values.

For arrays, this just means matching the value in the array. For the file `matching_array.yml` containing:
```
crabs:
  - abby
  - carl
  - alexandra
```
then:
```
ry matching_array.yml 'crabs(.==a*)' --printMode pv
```
will return:
```
crabs[0]: abby
crabs[2]: alexandra
```

For maps, this just means matching the map entry's value. For the file `matching_map.yml` containing:
```
crabs:
  name_1: abby
  name_2: carl
  name_3: alexandra
```
then:
```
ry matching_map.yml 'crabs(.==a*)' --printMode pv
```
will return:
```
crabs.name_1: abby
crabs.name_3: alexandra
```
Notice that this ignores the keys of the map entries.

### Length of filtered results
The length of filtered results get printed individually. For the file `test_filtered_length.yml` containing:
```
crabs:
  - abby
  - carl
  - alexandra
```
then:
```
ry test_filtered_length.yml 'crabs(.==a*)' --printMode pv --length
```
will return:
```
crabs[0]: 4
crabs[2]: 9
```

However, if you want to know the total number of filtered results, you can use the `--collect` and `--length` feature flags together. Then:
```
ry test_filtered_length.yml 'crabs(.==a*)' --length --collect
```
will return `2`.

---

## Benchmarking

These simple benchmarks were run using `ry` version 0.1.1 and `yq` version 3.2.1. They're searching an 80Mb yaml file `typeIDs.yaml` pulled from the [Eve Online Static Data Export](https://developers.eveonline.com/resource/resources).

[Hyperfine](https://github.com/sharkdp/hyperfine) was used for performing the benchmarks, with a warmup period of 3 runs and 10 benchmarking runs.

Two search patterns were tested:
- `'123*.name.en'`
    - both `ry` and `yq` report 55 matches on `typeIDs.yaml`
- `'12*.name.en'`
    - both `ry` and `yq` report 494 matches on `typeIDs.yaml`

The pattern `'*.name.en'` took more than 10 minutes for `yq`, so the search patterns had to be made more specific. The benchmarks were run on a 2.6 GHz Intel Core i7 processor with 32 GB of DRAM.

---

### ***'123\*.name.en'***

| Command | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| `ry typeIDs.yaml '123*.name.en'` | 2.923 ± 0.029 | 2.851 | 2.959 | 1.00 |

| Command | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| `yq r typeIDs.yaml '123*.name.en'` | 7.544 ± 0.203 | 7.440 | 8.097 | 1.00 |

---

### ***'12\*.name.en'***

| Command | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| `ry typeIDs.yaml '12*.name.en'` | 2.927 ± 0.035 | 2.890 | 2.993 | 1.00 |

| Command | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| `yq r typeIDs.yaml '12*.name.en'` | 46.173 ± 1.081 | 45.355 | 48.938 | 1.00 |

---

On `'123*.name.en'`, `ry` performs about 2.5x faster than `yq`, and on `'12*.name.en'`, `ry` performs about 15x faster than `yq`.