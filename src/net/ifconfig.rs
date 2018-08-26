use {Result, boxed::EfiBox, system_table, image_handle};
use alloc::Vec;
use core::{ptr, mem, slice};
use ffi::{
    EFI_HANDLE,
    VOID,
    EFI_BUFFER_TOO_SMALL,
    ip4::{
        EFI_IP4_SERVICE_BINDING_PROTOCOL_GUID,
        EFI_IP4_CONFIG_PROTOCOL_GUID,
        EFI_IP4_CONFIG_PROTOCOL,
        EFI_IP4_IPCONFIG_DATA,
        EFI_IP4_ROUTE_TABLE,
    },
    boot_services::{EFI_LOCATE_SEARCH_TYPE, EFI_OPEN_PROTOCOL_BY_HANDLE_PROTOCOL},
};
use net::addr::Ipv4Addr;

pub struct Interface {
    ipv4_config: EfiBox<EFI_IP4_IPCONFIG_DATA>,
    // TODO: add IPv6 config too
}

impl Interface {
    pub fn station_address_ipv4(&self) -> Ipv4Addr {
        self.ipv4_config.StationAddress.into()
    }

    pub fn subnet_mask_ipv4(&self) -> Ipv4Addr {
        self.ipv4_config.SubnetMask.into()
    }

    pub fn routes_ipv4(&self) -> Ipv4RouteTable {
        Ipv4RouteTable::from_raw_parts(self.ipv4_config.RouteTable, self.ipv4_config.RouteTableSize)
    }
}

pub struct Ipv4RouteTable<'a> { 
    routes: &'a [EFI_IP4_ROUTE_TABLE],
    curr_pos: usize,
}

impl<'a> Ipv4RouteTable<'a> {
    fn from_raw_parts(buf: *const EFI_IP4_ROUTE_TABLE, no_of_routes: u32) -> Self {
        Self {
            routes: unsafe { slice::from_raw_parts(buf, no_of_routes as usize) },
            curr_pos: 0,
        }
    }

    pub fn as_ptr(&self) -> *const EFI_IP4_ROUTE_TABLE {
        self.routes.as_ptr()
    }
}

impl<'a> Iterator for Ipv4RouteTable<'a> {
    type Item = Ipv4Route;
    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_pos < self.routes.len() {
            let route = Ipv4Route(self.routes[self.curr_pos].clone());
            self.curr_pos += 1;
            Some(route)
        } else {
            None
        }
    }
}

pub struct Ipv4Route(EFI_IP4_ROUTE_TABLE);

impl Ipv4Route {
    pub fn subnet_address(&self) -> Ipv4Addr {
        self.0.SubnetAddress.into()
    }

    pub fn subnet_mask(&self) -> Ipv4Addr {
        self.0.SubnetMask.into()
    }

    pub fn gateway_address(&self) -> Ipv4Addr {
        self.0.GatewayAddress.into()
    }
}

pub fn interfaces() -> Result<Vec<Interface>> {
    // TODO: should we not return an iterator instead of a vec here?
    let bs = system_table().BootServices;

    let mut no_of_handles = 0;
    let mut handle_buf: *const EFI_HANDLE = ptr::null_mut();
    unsafe {
      ret_on_err!(((*bs).LocateHandleBuffer)(EFI_LOCATE_SEARCH_TYPE::ByProtocol, &EFI_IP4_SERVICE_BINDING_PROTOCOL_GUID, ptr::null() as *const VOID, &mut no_of_handles, &mut handle_buf));
    }

    if no_of_handles == 0 || handle_buf.is_null() {
        return Ok(Vec::new());
    }

    let handle_buf = unsafe { EfiBox::from_raw(handle_buf as *mut EFI_HANDLE) };  // Putting it in a box for proper cleanup on exit

    let handles = unsafe { slice::from_raw_parts_mut(handle_buf.as_raw() as *mut EFI_HANDLE, no_of_handles) };
    
    // Enumerate all handles that installed with ip service binding protocol.
    let mut interfaces = Vec::new();
    for handle in handles.iter() {
        // config protocol and service binding protocol are installed on the same handle.
        let config_proto = ptr::null::<EFI_IP4_CONFIG_PROTOCOL>() as *const EFI_IP4_CONFIG_PROTOCOL;
        unsafe {
        ret_on_err!(((*bs).OpenProtocol)(*handle,
                    &EFI_IP4_CONFIG_PROTOCOL_GUID,
                    mem::transmute(&config_proto),
                    image_handle(),
                    ptr::null(),
                    EFI_OPEN_PROTOCOL_BY_HANDLE_PROTOCOL));
        }

        // TODO: add code to wait for IP protocol to initialize here.
        // Otherwise we get a no mapping error
        let mut data_size = 0;
        let status = unsafe { ((*config_proto).GetData)(config_proto, &mut data_size, ptr::null_mut()) };

        if status != EFI_BUFFER_TOO_SMALL {
            return Err(status.into());
        }

        let config_data = unsafe { EfiBox::<EFI_IP4_IPCONFIG_DATA>::allocate(data_size)? };
        unsafe { ret_on_err!(((*config_proto).GetData)(config_proto, &mut data_size, config_data.as_raw())); }

        interfaces.push(Interface { ipv4_config: config_data });
    }

    Ok(interfaces)
}