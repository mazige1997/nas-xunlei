#
# Copyright (C) 2015-2016 OpenWrt.org
#
# This is free software, licensed under the GNU General Public License v3.
#

include $(TOPDIR)/rules.mk

PKG_NAME:=nas-xunlei
PKG_VERSION:=3.5.2
PKG_RELEASE:=1
STRIP:=false

ifeq ($(ARCH),aarch64)
	BIN_HASH:=b65e56ca4749d0fab18f2a1c0d37edec9377205fe3d94033c2ac316557ea4f10
endif

ifeq ($(ARCH),x86_64)
	BIN_HASH:=8ecb2329c9a318d37e5c8a59bf9cfd71138df7e3fab91c888f0e7a30931bebaa
endif

PKG_MAINTAINER:=gngpp <gngppz@gmail.com>
PKG_BUILD_DIR:=$(BUILD_DIR)/$(PKG_NAME)-$(PKG_VERSION)
include $(INCLUDE_DIR)/package.mk

define Package/$(PKG_NAME)
  SECTION:=net
  CATEGORY:=Network
  SUBMENU:=Web Servers/Proxies
  TITLE:=nas-xunlei
  DEPENDS:=@(aarch64||x86_64) @USE_MUSL +libc6-compat +libgcc +libstdcpp
  URL:=https://nas.xunlei.com/
endef

define Package/$(PKG_NAME)/description
  Synology Nas Thunder runs on OpenWrt
endef

define Package/$(PKG_NAME)/postrm
#!/bin/sh
  rm -rf /usr/share/xunlei
  exit 0
endef

define Download/$(PKG_NAME)
  URL:=https://github.com/gngpp/nas-xunlei/releases/download/v$(PKG_VERSION)
  URL_FILE:=nas-xunlei-openwrt-$(ARCH)-$(PKG_VERSION).tar.xz
  FILE:=nas-xunlei-openwrt-$(ARCH)-$(PKG_VERSION).tar.xz
  HASH:=$(BIN_HASH)
endef

define Build/Prepare
	$(call Build/Prepare/Default)
	$(eval $(call Download,$(PKG_NAME)))
	$(TAR) -xf $(DL_DIR)/nas-xunlei-openwrt-$(ARCH)-$(PKG_VERSION).tar.xz -C $(PKG_BUILD_DIR)/
endef

define Build/Compile
endef

define Package/$(PKG_NAME)/install
	$(INSTALL_DIR) $(1)/var/packages/pan-xunlei-com/target
  $(INSTALL_DIR) $(1)/var/packages/pan-xunlei-com/target/host
	$(CP) $(PKG_BUILD_DIR)/bin $(1)/var/packages/pan-xunlei-com/target/
	$(INSTALL_BIN) $(PKG_BUILD_DIR)/xunlei $(1)/var/packages/pan-xunlei-com/xunlei

  $(INSTALL_DIR) $(1)/usr/syno/synoman/webman/modules
  $(INSTALL_DIR) $(1)/var/packages/pan-xunlei-com/target/host/etc
	$(INSTALL_DIR) $(1)/var/packages/pan-xunlei-com/target/host/usr/syno/synoman/webman/modules
endef

$(eval $(call BuildPackage,$(PKG_NAME)))