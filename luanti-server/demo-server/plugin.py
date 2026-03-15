import luanti

print("loading Python Plugin")

# ModchannelJoin, 0x17, Default, true => ModchannelJoinSpec,
def on_modchannel_join(channel_name: str):
    print(f"on_modchannel_join: {channel_name}")
    return

# ModchannelLeave, 0x18, Default, true => ModchannelLeaveSpec,
def on_modchannel_leave(channel_name: str):
    print(f"on_modchannel_leave: {channel_name}")
    return

# TSModchannelMsg, 0x19, Default, true => TSModchannelMsgSpec,
def on_ts_modchannel_msg(channel_name: str, channel_msg: str):
    print(f"on_ts_modchannel_msg: {channel_name}, {channel_msg}")
    return

# Playerpos, 0x23, Default, false => PlayerPosCommand,
def on_playerpos():
    print(f"on_playerpos …")
    return

# InventoryAction, 0x31, Default, true => InventoryActionSpec,
def on_inventory_action_move():
    print(f"on_inventory_action_move …")
    return

def on_inventory_action_craft():
    print(f"on_inventory_action_craft …")
    return

def on_inventory_action_drop():
    print(f"on_inventory_action_drop …")
    return

# TSChatMessage, 0x32, Default, true => TSChatMessageSpec,
def on_ts_chat_message(message: str):
    print(f"on_ts_chat_message: {message}")
    if message == "inv":
        luanti.inv()

# Damage, 0x35, Default, true => DamageSpec,
def on_damage(damage: int):
    print(f"on_damage: {damage}")
    return

# PlayerItem, 0x37, Default, true => PlayerItemSpec,
def on_player_item(item: int):
    luanti.fov(item / 10.0 + 1.0)
    print(f"on_player_item: {item}")
    return

# Respawn, 0x38, Default, true => RespawnSpec,
def on_respawn():
    print(f"on_respawn")
    return

# Interact, 0x39, Default, true => InteractSpec,
def on_interact():
    print(f"on_interact …")
    return

# NodemetaFields, 0x3b, Default, true => NodemetaFieldsSpec,
def on_nodemeta_fields():
    print(f"on_nodemeta_fields …")
    return

# InventoryFields, 0x3c, Default, true => InventoryFieldsSpec,
def on_inventory_fields():
    print(f"on_inventory_fields …")
    return
