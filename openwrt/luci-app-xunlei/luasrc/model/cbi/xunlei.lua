local m, s

m = Map("xunlei", translate("Xunlei"))
m.description = translate("<a>NAS Xunlei DSM 7.x Beta Version, Invitation code (3H9F7Y6D)</a> | <a href=\"https://github.com/gngpp/nas-xunlei\" target=\"_blank\">Project GitHub URL</a>")

m:section(SimpleSection).template = "xunlei/xunlei_status"

s = m:section(TypedSection, "xunlei")
s.addremove = false
s.anonymous = true

o = s:option(Flag, "enabled", translate("Enabled"))
o.rmempty = false

o = s:option(Flag, "internal", translate("Internal"))
o.rmempty = true

o = s:option(Value, "port", translate("Port"))
o.datatype = "and(port,min(1))"
o.default = "5051"
o.rmempty = false

o = s:option(Value, "config_path", translate("Data Storage Path"))
o.default = "/var/packages/pan-xunlei-com"

o = s:option(Value, "download_path", translate("Default Download Path"))
o.default = "/tmp/downloads"

return m
