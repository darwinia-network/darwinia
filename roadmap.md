# Land-Chain RoadMap
> Land-Chain(大陆链)作为进化星球中的一块大陆，将基于substrate开发。
> 


## [POC-1] basic token system
DEADLINE: 2019.6.1
功能包括：
 - 用ring抵押获得kton
 - 使用kton解锁抵押的ring
 - 抵押后的kton作为权益，可以(实时？)享受系统分成
 - kton分得的ring可以用来支付手续费(gas)
 - 如果kton分红不足以抵扣gas，则使用ring购买
 - 开发者可以代替用户付费（借鉴波场）
 - 

要求：
- 优先复用现有trait
- 优先解耦
- 进度优先，代码可以未来重构
- 

### 第一阶段： 2019.5.1前
    - [理论] 详解balances, staking模块
    - [实践] 增加kton模块，并完成用ring抵押获得kton功能

    - [理论] staking模块中的Phragmen算法
    - [X] 实现kton分成ring并抵扣gas

### 第二阶段： 2019.6.1前
        
1. 合约 
    1. 付费 （开发者付费/从抵押kton的分成优先付费）
    2. built-in account （系统分红）
    3. gas meter（目前有的方案）

## [POC-2] Staking
### 第三阶段：2019.7.1前
1. Staking
    1. validator
    2. collactor
    
        
 
        