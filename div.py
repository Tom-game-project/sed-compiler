# ================================================================
# sedで実装済みの、あるいは実装可能なプリミティブ関数
# ================================================================

# === 以前の実装 (メインではないため詳細は省略) ===
def add(a: str, b: str) -> str:
    if not a: return b
    if not b: return a
    return bin(int(a, 2) + int(b, 2))[2:]

def sub(a: str, b: str) -> str:
    # 2の補数を使った減算
    b_padded = b.zfill(len(a))
    b_not = "".join(['1' if c == '0' else '0' for c in b_padded])
    b_comp = add(b_not, "1")
    result = add(a, b_comp)
    return result[1:] if len(result) > len(a) else (result.lstrip('0') or '0')

def is_greater_or_equal(a: str, b: str) -> bool:
    a_norm = a.lstrip('0') or '0'
    b_norm = b.lstrip('0') or '0'
    if len(a_norm) > len(b_norm): return True
    if len(a_norm) < len(b_norm): return False
    return a_norm >= b_norm

def shift_left1(a: str) -> str:
    if a == "0": return "0"
    return a + "0"

# === 今回実装したヘルパー関数 ===
def is_empty(s: str) -> bool:
    return s == ""

def head(s: str) -> str:
    if is_empty(s): return ""
    return s[0]

def tail(s: str) -> str:
    if is_empty(s): return ""
    return s[1:]

def concat(s1: str, s2: str) -> str:
    return s1 + s2

# ================================================================
# 割り算と剰余の再帰実装
# ================================================================

def div_mod_rec(N_rem: str, Q: str, R: str, D: str) -> (str, str):
    # 終了条件（ベースケース）
    if is_empty(N_rem):
        return Q, R

    # 再帰ステップ
    B = head(N_rem)
    N_next = tail(N_rem)
    
    # 余り全体をシフトせず、単純にビットを連結する
    R_new = (R + B).lstrip('0') or '0'

    if is_greater_or_equal(R_new, D):
        R_next = sub(R_new, D)
        Q_next = concat(Q, "1")
    else:
        R_next = R_new
        Q_next = concat(Q, "0")
        
    return div_mod_rec(N_next, Q_next, R_next, D)

def div_mod(numerator: str, divisor: str) -> (str, str):
    if divisor == "0":
        raise ValueError("Cannot divide by zero.")
    if numerator == "0" or is_empty(numerator):
        return "0", "0"
    
    # 最初の余りの初期値は空文字列 "" で良い
    quotient, remainder = div_mod_rec(numerator, "", "", divisor)
    
    # 商の先頭の不要な0を取り除く
    return quotient.lstrip('0') or '0', remainder

# ================================================================
# 実行と検証
# ================================================================
if __name__ == "__main__":
    N = "11100100"  # 228
    D = "1010"      # 10
    
    print(f"div_mod({N}, {D}) を実行します...")
    
    q, r = div_mod(N, D)
    
    print(f"  商: {q} (10進数: {int(q, 2)})")
    print(f"  余り: {r} (10進数: {int(r, 2)})")

    assert q == bin(228 // 10)[2:]
    assert r == bin(228 % 10)[2:]
    print("\n✅ 検算OK")

