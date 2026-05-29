coroutine の場合と比べると、 async await は終了云々に対してブロックとか通知を入れる仕組みのように見える。
what color of your function の話があるので、関数は async と non async に分けられて、 non async は普通にやる。
async について、普通に呼ぶ場合はそのまま stack にのせたり、 run の場合は別の stack に分けたりするが、そこはどうやってもいい。
await をすると、その stack は普通に停止するが、coroutineでループするのと違って、どのタスクによってブロックされているのかで何を実行するのかを待つようにする。