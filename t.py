def rotate90(lst:list[str]) -> list[str]:
    # 各単語の n 文字目を、リストの後ろから順に結合して新しいリストを作成
    # (反時計回りに90度回転)

    return ["".join(chars) for chars in zip(*lst[::-1])]

def divide_and_3x3(lst:list[str]) -> list[str]:
    # 9x9の空のグリッドを用意
    grid = [['' for _ in range(9)] for _ in range(9)]

    for idx, word in enumerate(lst):
        # 単語を配置する3x3ブロックの左上の開始位置を計算
        block_row = (idx // 3) * 3
        block_col = (idx % 3) * 3

        # 3x3の形状になるように単語の文字を配置
        for i in range(3):
            for j in range(3):
                # 文字列からi行j列目の文字を取り出して配置
                grid[block_row + i][block_col + j] = word[i * 3 + j]
    return list(map(lambda a: "".join(a), grid))

word9_lst = [
    "Adventure",
    "Beautiful",
    "Challenge",
    "Different",
    "Education",
    "Furniture",
    "Knowledge",
    "Marketing",
    "Yesterday",
]

# 結果の表示
for word in divide_and_3x3(word9_lst):
    print(word)

# リスト形式での出力
# print(f"\nResult: {rotated_lst}")

# print("".join(rotate90(word9_lst)))
print("".join(divide_and_3x3(word9_lst)))
