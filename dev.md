# How to test

## soil_sample/decimal_add.soil

generate test case string

```sh
python3 t.py
# or
uv run t.py
```

compile test program

```sh
cargo run -p soilc -- -i soil_sample/decimal_add.soil -o out.sed
```

run program

```sh
# get_row
echo ~AdventureBeautifulChallengeDifferentEducationFurnitureKnowledgeMarketingYesterday~0 | sed -f out.sed

# get_column
echo ~YMKFEDCBAeanudihedsrorufaavtkwncflueeeliaeltnrtettreitdiduienfuangrongurygeentele~0 | sed -f out.sed

# get_square
echo ~AdvBeaChaentutilleurefulngeDifEduFurfercatnitentionureKnoMarYeswleketterdgeingday~0~1 | sed -f out.sed
```

```
012345678

0  1  2
AdvBeaCha 3 0
entutille   1
urefulnge   2
DifEduFur 6 3
fercatnit   4
entionure   5
KnoMarYes 9 6
wleketter   7
dgeingday   8
```

