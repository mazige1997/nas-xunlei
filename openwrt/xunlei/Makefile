#
# Copyright (C) 2015-2016 OpenWrt.org
#
# This is free software, licensed under the GNU General Public License v3.
#

include $(TOPDIR)/rules.mk

PKG_NAME:=xunlei
PKG_VERSION:=3.5.2
PKG_RELEASE:=1

PKG_LICENSE:=MIT
PKG_MAINTAINER:=gngpp <gngppz@gmail.com>

PKG_BUILD_DIR:=$(BUILD_DIR)/$(PKG_NAME)-$(PKG_VERSION)

PKG_SOURCE:=xunlei-openwrt-$(PKG_VERSION)-$(ARCH).tar.gz
PKG_SOURCE_URL:=https://github.com/gngpp/nas-xunlei/releases/download/v$(PKG_VERSION)/
PKG_HASH:=skip

include $(INCLUDE_DIR)/package.mk

define Package/$(PKG_NAME)
  SECTION:=net
  CATEGORY:=Network
  SUBMENU:=Web Servers/Proxies
  TITLE:=nas-xunlei
  DEPENDS:=@(aarch64||x86_64) @USE_MUSL
  URL:=https://nas.xunlei.com/
endef

define Package/$(PKG_NAME)/description
  Synology Nas Thunder runs on OpenWrt
endef

define Package/$(PKG_NAME)/conffiles
/etc/config/xunlei
endef

define Download/sha256sum
	FILE:=$(PKG_SOURCE).sha256
	URL_FILE:=$(FILE)
	URL:=$(PKG_SOURCE_URL)
	HASH:=skip
endef
$(eval $(call Download,sha256sum))

define Build/Prepare
	mv $(DL_DIR)/$(PKG_SOURCE).sha256 .
	cp $(DL_DIR)/$(PKG_SOURCE) .
	shasum -a 256 -c $(PKG_SOURCE).sha256
	rm $(PKG_SOURCE).sha256 $(PKG_SOURCE)

	tar -C $(PKG_BUILD_DIR)/ -zxf $(DL_DIR)/$(PKG_SOURCE)
endef

define Build/Compile
	echo "nas-xunlei using precompiled binary."
endef

define Package/$(PKG_NAME)/postrm
	#!/bin/sh
	rm -rf /var/packages/pan-xunlei-com
	exit 0
endef

define Package/$(PKG_NAME)/install
	$(INSTALL_DIR) $(1)/var/packages/pan-xunlei-com/target
	$(INSTALL_DIR) $(1)/var/packages/pan-xunlei-com/target/host
	$(CP) $(PKG_BUILD_DIR)/bin $(1)/var/packages/pan-xunlei-com/target/
	$(INSTALL_BIN) $(PKG_BUILD_DIR)/xunlei $(1)/var/packages/pan-xunlei-com/xunlei
	$(INSTALL_DIR) $(1)/usr/syno/synoman/webman/modules
	$(INSTALL_DIR) $(1)/var/packages/pan-xunlei-com/target/host/etc
	$(INSTALL_DIR) $(1)/var/packages/pan-xunlei-com/target/host/usr/syno/synoman/webman/modules
	$(INSTALL_BIN) $(CURDIR)/files/sysinfo.conf $(1)/var/packages/pan-xunlei-com/target/host/etc/sysinfo.conf
	echo "unique=\"synology_$(cat /proc/sys/kernel/random/uuid | cut -c1-7)_720+\"" > $(1)/var/packages/pan-xunlei-com/target/host/etc/sysinfo.conf
	$(INSTALL_BIN) $(CURDIR)/files/authenticate.cgi $(1)/var/packages/pan-xunlei-com/target/host/usr/syno/synoman/webman/modulesauthenticate.cgi
	$(INSTALL_DIR) $(1)/etc/init.d
	$(INSTALL_BIN) $(CURDIR)/files/xunlei.init $(1)/etc/init.d/xunlei
	$(INSTALL_DIR) $(1)/etc/config
	$(INSTALL_CONF) $(CURDIR)/files/xunlei.config $(1)/etc/config/xunlei
endef

$(eval $(call BuildPackage,$(PKG_NAME)))
