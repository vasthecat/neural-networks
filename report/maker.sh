#!/bin/sh

watch() {
    [ -z "$1" ] && echo "Необходимо указать название основного документа" && help
    set -o xtrace
    latexmk -pdf -f -shell-escape -interaction=nonstopmode -pvc $1
}

doc() {
    [ -z "$1" ] && echo "Необходимо указать название основного документа" && help
    set -o xtrace
    latexmk -pdf -f -shell-escape -interaction=nonstopmode $1
}

clean() {
    set -o xtrace
    rm -rf _minted-*
    find . -name "*.aux" -exec rm {} \;
    rm -f *.dvi *.fdb_latexmk *.fls *.log *.out *.toc
}

help() {
    echo "Использование:"
    echo "./maker.sh watch <main-doc>.tex -> Запуск процесса, пересобирающего документ при изменениях"
    echo "./maker.sh doc <main-doc>.tex -> Пересобрать документ"
    echo "./maker.sh clean -> Удаление сгенерированных файлов"
    exit 1
}

case "$1" in
    watch) watch $2 ;;
    doc) doc $2 ;;
    clean) clean ;;
    *) help ;;
esac
