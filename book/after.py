#!/usr/bin/env python3

# ビルド用のスクリプト
# mdbook build を行う。
# index.html を bin をもとに自動生成する。
# component を trunk を用いてビルドする。
# book/ の各mdにあたる out/book 下の html ファイルを探す。
# 各 html ファイルにある情報から、html ファイルを書き換えるなどし、wasm と js を移す

import os
import sys
import glob
import subprocess
import shutil
import re

def write_index_html_with_name(index_html, file_name):
    index_html.write("<!DOCTYPE html><html>\n")
    index_html.write("<head></head><body>\n")
    file_name_1 = os.path.split(file_name)[1]
    file_name_noext = os.path.splitext(file_name_1)[0]
    tag = "<link data-trunk rel=\"rust\" data-bin=\"" + file_name_noext + "\" data-type=\"main\"/>\n"
    index_html.write(tag)
    index_html.write("</body></html>\n")

# この python の実行中のディレクトリを python のある場所に設定する。
root_path = os.path.dirname(os.path.abspath(__file__))
book_path = os.path.join(root_path, "book")
book_src_path = os.path.join(book_path, "src")
comp_path = os.path.join(root_path, "component")
out_path = os.path.join(root_path, "out")
out_book_path = os.path.join(out_path, "book")
print("[INFO] cur-dir:", root_path)

# mdbook のビルドを行う。
os.chdir(book_path)
print("[INFO] mdbook building")
subprocess.run(["mdbook", "build"])
print("[INFO] mdbook build completed")

# 各 component のビルド成果物を取得する。
print("[INFO] trunk building")

## カレントディレクトリを book/component にする
os.chdir(comp_path)
## component 以下の dir を列挙する
for directory in os.listdir(comp_path):

    abs_path = os.path.join(comp_path, directory)
    print("[INFO] directory", abs_path)

    os.chdir(abs_path)

    ## directory/ にある名前を全て得て、確認する。
    target_reg = os.path.join(abs_path, "src/bin/*.rs")
    files = glob.glob(target_reg)
    print("[INFO] files list:")
    for file_name in files:
        print("-", file_name)

    ## directory/index.html を上で得た各ファイルごとに書き換え、trunkでビルドする。
    index_html_name = os.path.join(abs_path, "index.html")
    for file_name in files:
        print("[INFO] processing:", file_name)
        # index.html ファイルを上書きモードで開く。
        with open(index_html_name, mode='w') as index_html:
            # trunk 用に index.html を書く
            index_html.write("<!DOCTYPE html><html>\n")
            index_html.write("<head></head><body>\n")

            file_name_1 = os.path.split(file_name)[1]
            file_name_noext = os.path.splitext(file_name_1)[0]
            tag = "<link data-trunk rel=\"rust\" data-bin=\"" + file_name_noext + "\" data-type=\"main\"/>\n"
            index_html.write(tag)

            index_html.write("</body></html>\n")
        
        # trunk でビルドを行い得られたファイルを book/out/book に移動する
        os.chdir(abs_path)
        subprocess.run(["trunk", "build"])
        for file_name in glob.glob(os.path.join(out_path, "dist/*")):
            # index.html だけは除く。
            if os.path.split(file_name)[1] == "index.html":
                continue
            print("[INFO] copying file:", file_name)
            shutil.copy(file_name, out_book_path)
print("[INFO] trunk build completed")

# 得られたhtmlファイルに対して、wasm や js のロードを行うための変更を与える。
print("[INFO] html processing")
## ファイルの中身の文字列を得るためだけの関数
### // 変数のスコープやファイルをしっかり閉じているかどうかがpythonだとわかりにくいのでこれだけ別にした。
def file_str(file_name):
    file = open(file_name)
    content = file.read()
    file.close()
    return content

## ヘッドに追記する文字列を精製する関数
def load_head_html(name):
    head_load1 = '<link rel="preload" href="/' + name + '_bg.wasm" as="fetch" type="application/wasm" crossorigin="">'
    head_load2 = '<link rel="modulepreload" href="/'+ name + '.js">'
    return head_load1 + "\n" + head_load2 + "\n"

def load_body_html(name):
    body_load = '<script type="module">import init from \'/' + name + '.js\';init(\'/' + name + '_bg.wasm\');</script>'
    return body_load + "\n"

## カレントを book/book/src にする
os.chdir(book_src_path)
## <component id="X"> となっている文字列の検索のための正規表現
re_component = re.compile(r'<component id="(.*)">')
## html ファイルの分解を行うための正規表現
# re_html = re.compile(r'((.|$)*)<head>((.|$)*)</head>((.|$)*)<body>((.|$)*)</body>((.|$)*)')
re_html = re.compile(r'(.*)<head>(.*)</head>(.*)<body>(.*)</body>(.*)', flags=re.DOTALL)

### // 明らかに効率が悪いので後で直すこと
## 書き換えを行うhtmlファイルを全て列挙するため、mdファイルの名前を取得する。ただし、ディレクトリ構造を含めて名前を知りたい。
md_names = []
for candidate in glob.glob("./**/*.md", recursive=True):
    file_name_noext = os.path.splitext(os.path.basename(candidate))[0]
    if file_name_noext != "SUMMARY":
        md_names.append(candidate[:-3]) # 拡張子を除く
print("[INFO] find name:", md_names)

## カレントを out/book にする
os.chdir(out_book_path)

for md_name in md_names:
    file_name = os.path.join(out_book_path, md_name + ".html")
    print("[INFO] file:", file_name)

    content_html = file_str(file_name)
    # html の中にある <component> をすべて取得する。
    list_component = []
    for match_obj in re_component.finditer(content_html):
        list_component.append(match_obj.group(1))
    print("[INFO] component:", list_component)

    # html にある <component> を全て div に直したhtmlをえる。
    processed_html = re_component.sub(r'<div id="\1"></div>', content_html)

    # 追記を行う文字列を精製する。
    head_load_html = ""
    body_load_html = ""
    for name in list_component:
        head_load_html = head_load_html + load_head_html(name)
        body_load_html = body_load_html + load_body_html(name)

    # 得られたhtmlをhtmlの構造に合わせて正規表現で分解する。
    match_obj = re_html.search(processed_html)

    # 分解して得られた文字列をhead_loadやbody_loadを加えた文字列にする。
    head_html = "<head>" + match_obj.group(2) + head_load_html + "</head>"
    body_html = "<body>" + match_obj.group(4) + body_load_html + "</body>"
    result = match_obj.group(1) + head_html + match_obj.group(3) + body_html + match_obj.group(5)

    # 得られた文字列をファイルに書き込む。
    with open(file_name, mode='w') as file:
        file.write(result)

print("[INFO] process completed")

sys.exit(0)