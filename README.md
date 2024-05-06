令和元年度ネットワークスペシャリスト試験の午後１問３の「通信制限装置」を参考に作ったプログラムです。

ARPスプーフィングの原理を用いてEthernetフレームを中継し、送信元MACアドレスが登録済みであればプロキシを行いません。
ルーティング機能が無効な端末上で実行すれば自分宛てでないIPパケットはOS側でドロップしてくれるのでフィルタリングとして機能します。

## 機能
現在以下のような機能を実装しています。
- ARPプロキシの機能
- 特定のMACアドレスをARPプロキシの対象から外す機能
- Web APIから許可されたMACアドレスを登録／閲覧／削除する機能

## 設定方法
JSON形式の設定ファルを使用して設定を行います。コマンドライン引数で設定ファイルを指定する方式で実装予定です。
### コマンドライン引数
以下は`--help`の出力です。
```bash
Usage: arproxy-macfilter-agent [OPTIONS] <CONFIG_PATH>

Arguments:
  <CONFIG_PATH>  Path of configuration file (REQUIRED)

Options:
      --insecure  Accept insecure configuration
  -h, --help      Print help
  -V, --version   Print version
```
### 設定ファイルの形式
```json
{
    "interface":"lo",
    "allowed_mac_list": "/path/to/list.json",
    "arp_proxy": {
        "proxy_allowed_macs": false,
        "arp_reply_interval": 5,
        "arp_reply_duration": 60
    },
    "administration": {
        "enable_api": true,
        "listen_address": "127.0.0.1",
        "listen_port": 3000
    }
}
```
`administration.listen_address`にループバック以外のインターフェイスを設定すると警告が出ます。この警告を無視するにはコマンドライン引数`--insecure`を付けて実行する必要があります。
### APIによるホワイトリストの操作
**APIは認証機能を持ちません！**ループバックアドレスでリッスンするか、それも受け入れられない場合は `administration.enable_api` を `false` に設定してください。
`/api/allowed-mac` に許可されたMACアドレスの追加、取得、削除ができるAPIがあります。
```bash
# GET /api/allowed-mac/all 一覧表示
curl http://localhost/api/allowed-mac/all -s | jq
# POST /api/allowed-mac/add 追加
curl http://localhost/api/allowed-mac/add -X POST -H 'Content-Type: application/json' -d '{"mac_address":"02:00:00:00:00:01"}' -s | jq
# DELETE /api/allowed-mac/delete 削除
curl http://localhost/api/allowed-mac/delete -X DELETE -H 'Content-Type: application/json' -d '{"mac_address":"02:00:00:00:00:01"}' -s
```
### システムの設定
またこの通信制限装置の使用には前提条件としてシステムの設定を一部変更する必要があります。
#### IPフォワーディングの無効化
フィルタとして利用するにはIPフォワーディング機能を無効化する必要があります。カーネルパラメータを変更する方法が一番簡単です。
```bash
# システム全体で無効化する場合
sudo sysctl -w net.ipv4.ip_forward=0
# 特定のインターフェースのみ無効化する場合
sudo sysctl -w net.ipv4.conf.eth0.forwarding=0
```
なにか問題があって無効化できなかったら、iptablesやnftablesでPREROUTINGを全てDROPすると機能します。
#### その他カーネルパラメータの設定
動作に必須ではないと思いますがこのあたりの設定を行っておくと安定すると思います。
```bash
# ICMPリダイレクトを無効化
sudo sysctl -w net.ipv4.conf.all.send_redirects=0
```
## 実装予定（候補）の機能
以下のような機能の実装を現在検討しています。
### ネットワーク関連
- nftablesのsetに自動でMACアドレスを追加する機能
  - フォワーディングが有効なインターフェイス上でも使用できるように nftables でドロップする構成を取れるようにするといいかもしれません
- パフォーマンスの改善
  - セグメントのホスト数が増えると動作が安定しなくなります。
  - `nmap -sn`を実行するとわかりやすく動作が遅くなります。
### 管理機能
- 管理用コマンドラインアプリ
  - Web APIを叩くためのPythonスクリプトを用意するつもりです
- Web APIの認証機能
  - **現在認証機能を実装していません！**
  - ループバック以外のインターフェイスでも安全に利用できるように認証機能の実装を検討中です
- フロントエンド
  - 設定をブラウザから変更できるような機能を実装したいと思っています
