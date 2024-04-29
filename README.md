令和元年度ネットワークスペシャリスト試験の午後１問３の「通信制限装置」に着想を得て作ったプログラムです。

ARPスプーフィングの原理を用いてEthernetフレームを中継し、送信元MACアドレスが登録済みで開ければフレームをドロップします。

## 依存関係
- Linux
- sysctlパラメータ net.ipv4.ip_forward の有効化
- nftablesのインストール
その他にも細かな依存関係があるかもしれません。テストには以下の環境を使用しました。
```sh
$ uname -srvpio
Linux 6.5.0-28-generic #29~22.04.1-Ubuntu SMP PREEMPT_DYNAMIC Thu Apr  4 14:39:20 UTC 2 x86_64 x86_64 GNU/Linux
$ lsmod | grep nf_tables
nf_tables             372736  5
libcrc32c              12288  1 nf_tables
nfnetlink              20480  3 nfnetlink_queue,nf_tables,nfnetlink_log
$ sysctl net.ipv4.ip_forward
net.ipv4.ip_forward = 1
$ sysctl net.ipv4.conf.all.send_redirects
net.ipv4.conf.all.send_redirects = 0
```

## 機能
- ARPスプーフィングの機能
- nftables