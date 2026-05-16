# A Lark Bot for WeChat article collection

## 使用

首先需要在 [飞书开放平台](https://open.feishu.cn/app) 创建应用

填写基础信息后

**凭据与基础信息**: 保存好 `App ID` 和 `App Secret`

**添加应用能力**: 添加机器人能力

**权限管理**, 需要:

- im:message: 获取与发送单聊、群组消息
- im:message.group_at_msg:readonly: 获取群组中用户@机器人消息
- im:message.p2p_msg:readonly: 读取用户发给机器人的单聊消息
- im:message:send_as_bot: 以应用的身份发消息

**事件与回调 - 事件配置**

- 订阅方式: 选择 **长连接**
- 已添加事件
  - im.message.receive_v1: 接收消息, @与单发
