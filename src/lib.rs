mod internal;
mod errors;
use libc::ioctl;
pub use internal::{CmdContext, EthtoolCommnad, EthtoolLinkSettings};
pub use errors::EthtoolError;

const ETHTOOL_GLINKSETTINGS: u32 = 0x0000004C;

unsafe fn send_ioctl(mut ctx: CmdContext, ecmd: EthtoolCommnad) -> Result<CmdContext, EthtoolError> {
    ctx.update_ifr_from_ethtool_cmd(ecmd);
    let ret = ioctl(ctx.fd, libc::SIOCETHTOOL, &mut ctx.ifr as *mut _ as *mut libc::c_void);
    if ret == 0 {
        Ok(ctx)
    } else {
        Err(EthtoolError::new("Failed to send EthtoolCommnad"))
    }
}

pub unsafe fn do_ioctl_glinksettings(mut ctx: CmdContext) -> Result<EthtoolLinkSettings, EthtoolError> {
    let mut ecmd = EthtoolCommnad::new(ETHTOOL_GLINKSETTINGS)?;
	/* Handshake with kernel to determine number of words for link
	 * mode bitmaps. When requested number of bitmap words is not
	 * the one expected by kernel, the latter returns the integer
	 * opposite of what it is expecting. We request length 0 below
	 * (aka. invalid bitmap length) to get this info.
	 */
    ctx = send_ioctl(ctx, ecmd).expect("Failed to handshake with kernel");
    ecmd = ctx.get_ethtool_link_settings();
    if ecmd.req.link_mode_masks_nwords >= 0 || ecmd.req.cmd != ETHTOOL_GLINKSETTINGS {
        return Err(EthtoolError::new("Failed to determine number of words for link mode bitmaps"));
    }

	/* got the real ecmd.req.link_mode_masks_nwords,
	 * now send the real request
	 */
    ecmd.req.cmd = ETHTOOL_GLINKSETTINGS;
    ecmd.req.link_mode_masks_nwords = -ecmd.req.link_mode_masks_nwords;
    ctx = send_ioctl(ctx, ecmd).expect("Failed to get real request");
    ecmd = ctx.get_ethtool_link_settings();
    if ecmd.req.link_mode_masks_nwords <= 0 || ecmd.req.cmd != ETHTOOL_GLINKSETTINGS {
        return Err(EthtoolError::new("Failed to check the link_mode_masks_nwords."));
    }

    Ok(ecmd.req.clone())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let paths = std::fs::read_dir("/sys/class/net").unwrap();
        let mut ether_fibre_found = 0;
    
        for path in paths {
            if let Ok(entry) = path {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.starts_with("e") {
                        let ctx = CmdContext::new(file_name).unwrap();
                        let link_settings = unsafe { do_ioctl_glinksettings(ctx).unwrap() };
                        let bash_cmd = format!("ethtool {} | grep -q 'Port: Fibre'", file_name);
                        let ethtool_check_ret = std::process::Command::new("bash")
                            .arg("-c")
                            .arg(&bash_cmd)
                            .output()
                            .expect("Failed to execute command");

                        if ethtool_check_ret.status.success() {
                            ether_fibre_found = 1;
                            assert_eq!(link_settings.port, 3);
                        }
                    }
                }
            }
        }
        assert_eq!(ether_fibre_found, 1);
    }
}
