use libc::{c_int, ifreq, socket};
use std::mem::{zeroed, transmute};
use crate::errors::EthtoolError;

/// Context for sub-commands
#[repr(C)]
#[derive(Clone)]
pub struct CmdContext {
    devname: *const i8,
    pub fd: c_int,
    pub ifr: ifreq,
    /*
    argc: c_uint,
    argp: *const *const i8,
    debug: c_ulong,
    json: bool,
    show_stats: bool
    */
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct EthtoolLinkSettings {
    pub cmd: u32,
    pub speed: u32,
    pub duplex: u8,
    pub port: u8,
    pub phy_address: u8,
    pub autoneg: u8,
    pub mdio_support: u8,
    pub eth_tp_mdix: u8,
    pub eth_tp_mdix_ctrl: u8,
    pub link_mode_masks_nwords: i8,
    pub reserved1: [u8; 3],
    pub reserved: [u32; 7],
    pub link_mode_masks: [u32; 3 * 128],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct EthtoolCommnad {
    pub req: EthtoolLinkSettings,
    link_mode_data: [u32; 3 * 128]
}

impl EthtoolCommnad {
    pub fn new(cmd: u32) -> Result<Self, EthtoolError> {
        unsafe {
            let mut ecmd: Self = zeroed();
            ecmd.req.cmd = cmd;
            Ok(ecmd)
        }
    }
}


impl CmdContext {
    pub fn new(dev_name: &str) -> Result<Self, EthtoolError> {
        unsafe {
            let mut cmd_context: CmdContext = zeroed();
            if dev_name.len() > 16 {
                return Err(EthtoolError::new("The device name is too long."));
            }

            let mut dev_name_arr = [0u8; 16];
            dev_name_arr[..dev_name.len()].copy_from_slice(dev_name.as_bytes());

            let mut socket_fd = socket(libc::AF_INET, libc::SOCK_DGRAM, 0);
            if socket_fd < 0 {
                socket_fd = socket(libc::AF_NETLINK, libc::SOCK_RAW, libc::NETLINK_GENERIC);
            };
            if socket_fd < 0 {
                return Err(EthtoolError::new("Can't create socket control."))
            }

            cmd_context.devname = dev_name_arr.map(|x| x as i8).as_ptr();
            cmd_context.fd = socket_fd;
            Ok(cmd_context)
        }
    }

    fn update_ifr(&mut self, ifr: ifreq) {
        self.ifr = ifr
    }

    pub fn get_ethtool_link_settings(&self) -> EthtoolCommnad {
        unsafe {
            let ecmd = self.ifr.ifr_ifru.ifru_data as *mut EthtoolCommnad;
            *ecmd
        }
    }

    pub fn update_ifr_from_ethtool_cmd(&mut self, mut ecmd: EthtoolCommnad) {
        unsafe {
            let mut dev_name_arr = [0i8; 16];
            let dev_name_arr_i8: &[i8] = std::slice::from_raw_parts(self.devname, 16);
            dev_name_arr.copy_from_slice(dev_name_arr_i8);
            let ifreq = ifreq {
                ifr_name: dev_name_arr,
                ifr_ifru: libc::__c_anonymous_ifr_ifru {
                    ifru_data: transmute(&mut ecmd as *mut _),
                },
            };
            self.update_ifr(ifreq)
        }
    }
}
