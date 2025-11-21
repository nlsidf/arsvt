ａｒｓｖｔ３ｄ

在线demo：https://sage-pithivier-29c3c8.netlify.app/

快速本地体验：'npm install arsvt3d && arsvt3d'

![图片](https://github.com/nlsidf/arsvt/blob/main/photos/游戏截图3.png)

这份rust代码是伪3d渲染迷宫，应用ASCII字符画风格和tui技术，在保证效果的基础上尽可能节约资源//


使用了ratrtui库和crossterm库，实现跨平台架构运行，使用rand库的随机算法生成迷宫确保新奇感的体验//


在services文件夹是我用rust重构的ttyd项目的程序，同样使用了xterm.js，可以将本tui程序到浏览器中运行，但这个services（ttyd-rust）程序目前只支持linux版本部署//


添加了可以在墙壁移动的npc：“T^T”和“（^_^）”//


以及墙壁上的菱形符号//

![图片](https://github.com/nlsidf/arsvt/blob/main/photos/游戏截图2.png)

游戏中没有设置迷宫终点，请尽情探索//


推荐在linux环境或win10/11下运行（powershell版本大于等于7）//


或者使用tty web这类webterminal工具将其放到浏览器中使用，更加灵活//


下载仓库后cargo run --release 运行//


或者直接使用cargo install --git https://gitee.com/nlsidf/arsvt.git/


或者打开pocjet文件夹中的html文件（非本项目核心）体验在浏览器中效果//

![photo](https://github.com/nlsidf/arsvt/blob/main/photos/游戏截图4.png/)

仓库中有在Windows10上预编译的程序//


还有在termux上编译的适用于Android arm64的版本//

![图](https://github.com/nlsidf/arsvt/blob/main/photos/arsvt3d.png)

移动：wasd或箭头↑↓或屏幕按钮//


视角转换：鼠标拖拽或→←或按钮//


点击energy条可以切换纯色（白）模式或彩色模式，默认彩色//


点击小地图可以全屏3d view//


全屏后点击小地图之前所在的位置退出全屏//


e/c键：向上/向下转动视角//


空格键：跳跃//


m键：切换彩色纯色模式//


esc键退出程序//


将这个代码喂给ai能辅助我们完成3d场景的实现//


如pocket文件夹中的html程序就是我把这个项目喂给ai，实现另外的3d效果，减弱ai实现3d游戏或场景的幻觉//


本项目使用了ai agent cli/ide，但也是我用termux在手机上一点点敲的代码，不容易啊//


就到这里吧，感谢使用与分享//

![图片](https://github.com/nlsidf/arsvt/blob/main/photos/%E6%B8%B8%E6%88%8F%E6%88%AA%E5%9B%BE.png)

yanzicheng //


lglfr-nlsidf//

