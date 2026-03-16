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
echo ~YMKFEDCBAeanudihedsrorufaavtkwncflueeeliaeltnrtettreitdiduienfuangrongurygeentele~0 | sed -f out.sed
```

