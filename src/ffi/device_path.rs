use ffi::base::{
    EFI_GUID,
    EFI_MAC_ADDRESS,
    EFI_PHYSICAL_ADDRESS,
    EFI_IPv4_ADDRESS,
    EFI_IPv6_ADDRESS,
    CHAR8,
    CHAR16,
    UINT8,
    UINT16,
    UINT32,
    UINT64,
    UINTN,
    BOOLEAN,
    NOT_DEFINED};


pub const EFI_DEVICE_PATH_UTILITIES_PROTOCOL_GUID: EFI_GUID = EFI_GUID(0x379be4e, 0xd706, 0x437d, [0xb0, 0x37, 0xed, 0xb8, 0x2f, 0xb7, 0x72, 0xa4]);

#[repr(C)]
pub struct EFI_DEVICE_PATH_UTILITIES_PROTOCOL {
    pub GetDevicePathSize: EFI_DEVICE_PATH_UTILS_GET_DEVICE_PATH_SIZE, 
    pub DuplicateDevicePath: EFI_DEVICE_PATH_UTILS_DUP_DEVICE_PATH,
    pub AppendDevicePath: EFI_DEVICE_PATH_UTILS_APPEND_PATH,
    pub AppendDeviceNode: EFI_DEVICE_PATH_UTILS_APPEND_NODE,
    pub AppendDevicePathInstance: EFI_DEVICE_PATH_UTILS_APPEND_INSTANCE,
    pub GetNextDevicePathInstance: EFI_DEVICE_PATH_UTILS_GET_NEXT_INSTANCE,
    pub IsDevicePathMultiInstance: EFI_DEVICE_PATH_UTILS_IS_MULTI_INSTANCE,
    pub CreateDeviceNode: EFI_DEVICE_PATH_UTILS_CREATE_NODE,
}

pub type EFI_DEVICE_PATH_UTILS_GET_DEVICE_PATH_SIZE = *const NOT_DEFINED;
pub type EFI_DEVICE_PATH_UTILS_DUP_DEVICE_PATH = *const NOT_DEFINED;
pub type EFI_DEVICE_PATH_UTILS_APPEND_INSTANCE = *const NOT_DEFINED;
pub type EFI_DEVICE_PATH_UTILS_GET_NEXT_INSTANCE = *const NOT_DEFINED;
pub type EFI_DEVICE_PATH_UTILS_IS_MULTI_INSTANCE = *const NOT_DEFINED;

pub type EFI_DEVICE_PATH_UTILS_APPEND_PATH = extern "win64" fn(
    Src1: *const EFI_DEVICE_PATH_PROTOCOL,
    Src2: *const EFI_DEVICE_PATH_PROTOCOL
) -> *const EFI_DEVICE_PATH_PROTOCOL;

pub type EFI_DEVICE_PATH_UTILS_APPEND_NODE = extern "win64" fn(
    DevicePath: *const EFI_DEVICE_PATH_PROTOCOL,
    DevicePath: *const EFI_DEVICE_PATH_PROTOCOL
) -> *const EFI_DEVICE_PATH_PROTOCOL;

pub type EFI_DEVICE_PATH_UTILS_CREATE_NODE = extern "win64" fn(
    NodeType: UINT8,
    NodeSubType: UINT8,
    NodeLength: UINT16
) -> *const EFI_DEVICE_PATH_PROTOCOL;



pub const EFI_DEVICE_PATH_PROTOCOL_GUID: EFI_GUID = EFI_GUID(0x09576e91, 0x6d3f, 0x11d2, [0x8e, 0x39, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b]);

// Almost all the structs here are unaligned (repr(packed)) because UEFI spec wants it that way
#[derive(Debug)]
#[repr(packed)]
pub struct EFI_DEVICE_PATH_PROTOCOL {
    pub Type: UINT8,
    pub SubType: UINT8,
    pub Length: [UINT8; 2]
}


pub const HARDWARE_DEVICE_PATH: UINT8 = 0x01; 

pub const HW_PCI_DP: UINT8 = 0x01;

#[repr(packed)]
pub struct PCI_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  pub Function: UINT8,
  pub Device: UINT8,
} 


pub const HW_PCCARD_DP: UINT8 = 0x02;

#[repr(packed)]
pub struct PCCARD_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  pub FunctionNumber: UINT8,
}


pub const HW_MEMMAP_DP: UINT8 = 0x03;

#[repr(packed)]
pub struct MEMMAP_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  pub MemoryType: UINT32,
  pub StartingAddress: EFI_PHYSICAL_ADDRESS,
  pub EndingAddress: EFI_PHYSICAL_ADDRESS,
}

pub const HW_VENDOR_DP: UINT8 = 0x04;

#[repr(packed)]
pub struct VENDOR_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  pub Guid: EFI_GUID,
  // Vendor-defined variable size data comes here.
}

pub const HW_CONTROLLER_DP: UINT8 = 0x05;

#[repr(packed)]
pub struct CONTROLLER_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  pub ControllerNumber: UINT32,
}


pub const ACPI_DEVICE_PATH: UINT8 = 0x02;
pub const ACPI_DP: UINT8  = 0x01;

#[repr(packed)]
pub struct ACPI_HID_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  pub HID: UINT32,
  pub UID: UINT32,
}


pub const ACPI_EXTENDED_DP: UINT8 = 0x02;

#[repr(packed)]
pub struct ACPI_EXTENDED_HID_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  pub HID: UINT32,
  pub UID: UINT32,
  pub CID: UINT32,
  // Optional variable length _HIDSTR here
  // Optional variable length _UIDSTR here
  // Optional variable length _CIDSTR here
}

pub const MESSAGING_DEVICE_PATH: UINT8 = 0x03;

pub const MSG_ATAPI_DP: UINT8 = 0x01;

#[repr(packed)]
pub struct  ATAPI_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  ///
  /// Set to zero for primary, or one for secondary.
  ///
  pub PrimarySecondary: UINT8,
  ///
  /// Set to zero for master, or one for slave mode.
  ///
  pub SlaveMaster: UINT8,
  ///
  /// Logical Unit Number.
  ///
  pub Lun: UINT16,
}

pub const MSG_SCSI_DP: UINT8 = 0x02;

#[repr(packed)]
pub struct SCSI_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  ///
  /// Target ID on the SCSI bus (PUN).
  ///
  pub Pun: UINT16,
  ///
  /// Logical Unit Number (LUN).
  ///
  pub Lun: UINT16,
}

pub const MSG_FIBRECHANNEL_DP: UINT8 = 0x03;

#[repr(packed)]
pub struct FIBRECHANNEL_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  ///
  /// Reserved for the future.
  ///
  pub Reserved: UINT32,
  ///
  /// Fibre Channel World Wide Number.
  ///
  pub WWN: UINT64,
  ///
  /// Fibre Channel Logical Unit Number.
  ///
  pub Lun: UINT64,
}

pub const MSG_FIBRECHANNELEX_DP: UINT8 = 0x15;

#[repr(packed)]
pub struct FIBRECHANNELEX_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  ///
  /// Reserved for the future.
  ///
  pub Reserved: UINT32,
  ///
  /// 8 byte array containing Fibre Channel End Device Port Name.
  ///
  pub WWN: [UINT8;8],
  ///
  /// 8 byte array containing Fibre Channel Logical Unit Number.
  ///
  pub Lun: [UINT8;8],
}

pub const MSG_1394_DP: UINT8 = 0x04;

#[repr(packed)]
pub struct F1394_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  ///
  /// Reserved for the future.
  ///
  pub Reserved: UINT32,
  ///
  /// 1394 Global Unique ID (GUID).
  ///
  pub Guid: UINT64,
}

pub const MSG_USB_DP: UINT8 = 0x05;

#[repr(packed)]
pub struct USB_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  ///
  /// USB Parent Port Number.
  ///
  pub ParentPortNumber: UINT8,
  ///
  /// USB Interface Number.
  ///
  pub InterfaceNumber: UINT8,
}

pub const MSG_USB_CLASS_DP: UINT8 = 0x0f;

#[repr(packed)]
pub struct USB_CLASS_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  ///
  /// Vendor ID assigned by USB-IF. A value of 0xFFFF will
  /// match any Vendor ID.
  ///
  pub VendorId: UINT16,
  ///
  /// Product ID assigned by USB-IF. A value of 0xFFFF will
  /// match any Product ID.
  ///
  pub ProductId: UINT16,
  ///
  /// The class code assigned by the USB-IF. A value of 0xFF
  /// will match any class code.
  ///
  pub DeviceClass: UINT8,
  ///
  /// The subclass code assigned by the USB-IF. A value of
  /// 0xFF will match any subclass code.
  ///
  pub DeviceSubClass: UINT8,
  ///
  /// The protocol code assigned by the USB-IF. A value of
  /// 0xFF will match any protocol code.
  ///
  pub DeviceProtocol: UINT8,
}

pub const MSG_USB_WWID_DP: UINT8 = 0x10;

///
/// This device path describes a USB device using its serial number.
///
#[repr(packed)]
pub struct USB_WWID_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  ///
  /// USB interface number.
  ///
  pub InterfaceNumber: UINT16,
  ///
  /// USB vendor id of the device.
  ///
  pub VendorId: UINT16,
  ///
  /// USB product id of the device.
  ///
  pub ProductId: UINT16,
  //
  // Last 64-or-fewer UTF-16 characters of the USB
  // serial number. The length of the string is
  // determined by the Length field less the offset of the
  // Serial Number field (10)
  //
  // CHAR16                     SerialNumber[...];
}

///
/// Device Logical Unit SubType.
///
pub const MSG_DEVICE_LOGICAL_UNIT_DP: UINT8 = 0x11;

#[repr(packed)]
pub struct DEVICE_LOGICAL_UNIT_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  ///
  /// Logical Unit Number for the interface.
  ///
  pub Lun: UINT8,
}

///
/// SATA Device Path SubType.
///
pub const MSG_SATA_DP: UINT8 = 0x12;

#[repr(packed)]
pub struct SATA_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  ///
  /// The HBA port number that facilitates the connection to the
  /// device or a port multiplier. The value 0xFFFF is reserved.
  ///
  pub HBAPortNumber: UINT16,
  ///
  /// The Port multiplier port number that facilitates the connection
  /// to the device. Bit 15 should be set if the device is directly
  /// connected to the HBA.
  ///
  pub PortMultiplierPortNumber: UINT16,
  ///
  /// Logical Unit Number.
  ///
  pub Lun: UINT16,
}

///
/// Flag for if the device is directly connected to the HBA.
///
pub const SATA_HBA_DIRECT_CONNECT_FLAG: UINTN = 0x8000;

///
/// I2O Device Path SubType.
///
pub const MSG_I2O_DP: UINT8 = 0x06;

#[repr(packed)]
pub struct I2O_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  ///
  /// Target ID (TID) for a device.
  ///
  pub Tid: UINT32,
}

///
/// MAC Address Device Path SubType.
///
pub const MSG_MAC_ADDR_DP: UINT8 = 0x0b;

#[repr(packed)]
pub struct MAC_ADDR_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  ///
  /// The MAC address for a network interface padded with 0s.
  ///
  pub MacAddress: EFI_MAC_ADDRESS,
  ///
  /// Network interface type(i.e. 802.3, FDDI).
  ///
  pub IfType: UINT8,
}

///
/// IPv4 Device Path SubType
///
#[allow(non_upper_case_globals)]
pub const MSG_IPv4_DP: UINT8  = 0x0c;

#[repr(packed)]
pub struct IPv4_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  ///
  /// The local IPv4 address.
  ///
  pub LocalIpAddress: EFI_IPv4_ADDRESS,
  ///
  /// The remote IPv4 address.
  ///
  pub RemoteIpAddress: EFI_IPv4_ADDRESS,
  ///
  /// The local port number.
  ///
  pub LocalPort: UINT16,
  ///
  /// The remote port number.
  ///
  pub RemotePort: UINT16,
  ///
  /// The network protocol(i.e. UDP, TCP).
  ///
  pub Protocol: UINT16,
  ///
  /// 0x00 - The Source IP Address was assigned though DHCP.
  /// 0x01 - The Source IP Address is statically bound.
  ///
  pub StaticIpAddress: BOOLEAN,
  ///
  /// The gateway IP address
  ///
  pub GatewayIpAddress: EFI_IPv4_ADDRESS,
  ///
  /// The subnet mask
  ///
  pub SubnetMask: EFI_IPv4_ADDRESS,
}

///
/// IPv6 Device Path SubType.
///
#[allow(non_upper_case_globals)]
pub const MSG_IPv6_DP: UINT8 = 0x0d;

#[repr(packed)]
pub struct IPv6_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  ///
  /// The local IPv6 address.
  ///
  pub LocalIpAddress: EFI_IPv6_ADDRESS,
  ///
  /// The remote IPv6 address.
  ///
  pub RemoteIpAddress: EFI_IPv6_ADDRESS,
  ///
  /// The local port number.
  ///
  pub LocalPort: UINT16,
  ///
  /// The remote port number.
  ///
  pub RemotePort: UINT16,
  ///
  /// The network protocol(i.e. UDP, TCP).
  ///
  pub Protocol: UINT16,
  ///
  /// 0x00 - The Local IP Address was manually configured.
  /// 0x01 - The Local IP Address is assigned through IPv6
  /// stateless auto-configuration.
  /// 0x02 - The Local IP Address is assigned through IPv6
  /// stateful configuration.
  ///
  pub IpAddressOrigin: UINT8,
  ///
  /// The prefix length
  ///
  pub PrefixLength: UINT8,
  ///
  /// The gateway IP address
  ///
  pub GatewayIpAddress: EFI_IPv6_ADDRESS,
}

///
/// InfiniBand Device Path SubType.
///
pub const MSG_INFINIBAND_DP: UINT8 = 0x09;

#[repr(packed)]
pub struct INFINIBAND_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  ///
  /// Flags to help identify/manage InfiniBand device path elements:
  /// Bit 0 - IOC/Service (0b = IOC, 1b = Service).
  /// Bit 1 - Extend Boot Environment.
  /// Bit 2 - Console Protocol.
  /// Bit 3 - Storage Protocol.
  /// Bit 4 - Network Protocol.
  /// All other bits are reserved.
  ///
  pub ResourceFlags: UINT32,
  ///
  /// 128-bit Global Identifier for remote fabric port.
  ///
  pub PortGid: [UINT8;16],
  ///
  /// 64-bit unique identifier to remote IOC or server process.
  /// Interpretation of field specified by Resource Flags (bit 0).
  ///
  pub ServiceId: UINT64,
  ///
  /// 64-bit persistent ID of remote IOC port.
  ///
  pub TargetPortId: UINT64,
  ///
  /// 64-bit persistent ID of remote device.
  ///
  pub DeviceId: UINT64,
}

pub const INFINIBAND_RESOURCE_FLAG_IOC_SERVICE: UINTN = 0x01;
pub const INFINIBAND_RESOURCE_FLAG_EXTENDED_BOOT_ENVIRONMENT: UINTN = 0x02;
pub const INFINIBAND_RESOURCE_FLAG_CONSOLE_PROTOCOL: UINTN = 0x04;
pub const INFINIBAND_RESOURCE_FLAG_STORAGE_PROTOCOL: UINTN = 0x08;
pub const INFINIBAND_RESOURCE_FLAG_NETWORK_PROTOCOL: UINTN = 0x10;

///
/// UART Device Path SubType.
///
pub const MSG_UART_DP: UINT8 = 0x0e;

#[repr(packed)]
pub struct UART_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  ///
  /// Reserved.
  ///
  pub Reserved: UINT32,
  ///
  /// The baud rate setting for the UART style device. A value of 0
  /// means that the device's default baud rate will be used.
  ///
  pub BaudRate: UINT64,
  ///
  /// The number of data bits for the UART style device. A value
  /// of 0 means that the device's default number of data bits will be used.
  ///
  pub DataBits: UINT8,
  ///
  /// The parity setting for the UART style device.
  /// Parity 0x00 - Default Parity.
  /// Parity 0x01 - No Parity.
  /// Parity 0x02 - Even Parity.
  /// Parity 0x03 - Odd Parity.
  /// Parity 0x04 - Mark Parity.
  /// Parity 0x05 - Space Parity.
  ///
  pub Parity: UINT8,
  ///
  /// The number of stop bits for the UART style device.
  /// Stop Bits 0x00 - Default Stop Bits.
  /// Stop Bits 0x01 - 1 Stop Bit.
  /// Stop Bits 0x02 - 1.5 Stop Bits.
  /// Stop Bits 0x03 - 2 Stop Bits.
  ///
  pub StopBits: UINT8,
}

//
// Use VENDOR_DEVICE_PATH struct
//
pub const MSG_VENDOR_DP: UINT8 = 0x0a;
pub type VENDOR_DEFINED_DEVICE_PATH = VENDOR_DEVICE_PATH;

// #define DEVICE_PATH_MESSAGING_PC_ANSI     EFI_PC_ANSI_GUID
// #define DEVICE_PATH_MESSAGING_VT_100      EFI_VT_100_GUID
// #define DEVICE_PATH_MESSAGING_VT_100_PLUS EFI_VT_100_PLUS_GUID
// #define DEVICE_PATH_MESSAGING_VT_UTF8     EFI_VT_UTF8_GUID

///
/// A new device path node is defined to declare flow control characteristics.
/// UART Flow Control Messaging Device Path
///

#[repr(packed)]
pub struct UART_FLOW_CONTROL_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  ///
  /// DEVICE_PATH_MESSAGING_UART_FLOW_CONTROL GUID.
  ///
  pub Guid: EFI_GUID,
  ///
  /// Bitmap of supported flow control types.
  /// Bit 0 set indicates hardware flow control.
  /// Bit 1 set indicates Xon/Xoff flow control.
  /// All other bits are reserved and are clear.
  ///
  pub FlowControlMap: UINT32,
}

pub const UART_FLOW_CONTROL_HARDWARE: UINTN = 0x00000001;
pub const UART_FLOW_CONTROL_XON_XOFF: UINTN = 0x00000010;

// #define DEVICE_PATH_MESSAGING_SAS          EFI_SAS_DEVICE_PATH_GUID
///
/// Serial Attached SCSI (SAS) Device Path.
///

#[repr(packed)]
pub struct SAS_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  ///
  /// DEVICE_PATH_MESSAGING_SAS GUID.
  ///
  pub Guid: EFI_GUID,
  ///
  /// Reserved for future use.
  ///
  pub Reserved: UINT32,
  ///
  /// SAS Address for Serial Attached SCSI Target.
  ///
  pub SasAddress: UINT64,
  ///
  /// SAS Logical Unit Number.
  ///
  pub Lun: UINT64,
  ///
  /// More Information about the device and its interconnect.
  ///
  pub DeviceTopology: UINT16,
  ///
  /// Relative Target Port (RTP).
  ///
  pub RelativeTargetPort: UINT16,
}

///
/// Serial Attached SCSI (SAS) Ex Device Path SubType
///
pub const MSG_SASEX_DP: UINT8 = 0x16;

#[repr(packed)]
pub struct SASEX_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  ///
  /// 8-byte array of the SAS Address for Serial Attached SCSI Target Port.
  ///
  pub SasAddress: [UINT8;8],
  ///
  /// 8-byte array of the SAS Logical Unit Number.
  ///
  pub Lun: [UINT8;8],
  ///
  /// More Information about the device and its interconnect.
  ///
  pub DeviceTopology: UINT16,
  ///
  /// Relative Target Port (RTP).
  ///
  pub RelativeTargetPort: UINT16,
}

///
/// NvmExpress Namespace Device Path SubType.
///
pub const MSG_NVME_NAMESPACE_DP: UINT8 = 0x17;

#[repr(packed)]
pub struct NVME_NAMESPACE_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  pub NamespaceId: UINT32,
  pub NamespaceUuid: UINT64,
}

///
/// iSCSI Device Path SubType
///
pub const MSG_ISCSI_DP: UINT8 = 0x13;

#[repr(packed)]
pub struct ISCSI_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  ///
  /// Network Protocol (0 = TCP, 1+ = reserved).
  ///
  pub NetworkProtocol: UINT16,
  ///
  /// iSCSI Login Options.
  ///
  pub LoginOption: UINT16,
  ///
  /// iSCSI Logical Unit Number.
  ///
  pub Lun: UINT64,
  ///
  /// iSCSI Target Portal group tag the initiator intends
  /// to establish a session with.
  ///
  pub TargetPortalGroupTag: UINT16,
  //
  // iSCSI NodeTarget Name. The length of the name
  // is determined by subtracting the offset of this field from Length.
  //
  // CHAR8                        iSCSI Target Name.
}

pub const ISCSI_LOGIN_OPTION_NO_HEADER_DIGEST: UINTN = 0x0000;
pub const ISCSI_LOGIN_OPTION_HEADER_DIGEST_USING_CRC32C: UINTN = 0x0002;
pub const ISCSI_LOGIN_OPTION_NO_DATA_DIGEST: UINTN = 0x0000;
pub const ISCSI_LOGIN_OPTION_DATA_DIGEST_USING_CRC32C: UINTN = 0x0008;
pub const ISCSI_LOGIN_OPTION_AUTHMETHOD_CHAP: UINTN = 0x0000;
pub const ISCSI_LOGIN_OPTION_AUTHMETHOD_NON: UINTN = 0x1000;
pub const ISCSI_LOGIN_OPTION_CHAP_BI: UINTN = 0x0000;
pub const ISCSI_LOGIN_OPTION_CHAP_UNI: UINTN = 0x2000;

///
/// VLAN Device Path SubType.
///
pub const MSG_VLAN_DP: UINT8 = 0x14;

#[repr(packed)]
pub struct VLAN_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  ///
  /// VLAN identifier (0-4094).
  ///
  pub VlanId: UINT16,
}

//
// Media Device Path
//
pub const MEDIA_DEVICE_PATH: UINT8 = 0x04;

///
/// Hard Drive Media Device Path SubType.
///
pub const MEDIA_HARDDRIVE_DP: UINT8 = 0x01;

///
/// The Hard Drive Media Device Path is used to represent a partition on a hard drive.
///
#[repr(packed)]
pub struct HARDDRIVE_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  ///
  /// Describes the entry in a partition table, starting with entry 1.
  /// Partition number zero represents the entire device. Valid
  /// partition numbers for a MBR partition are [1, 4]. Valid
  /// partition numbers for a GPT partition are [1, NumberOfPartitionEntries].
  ///
  pub PartitionNumber: UINT32,
  ///
  /// Starting LBA of the partition on the hard drive.
  ///
  pub PartitionStart: UINT64,
  ///
  /// Size of the partition in units of Logical Blocks.
  ///
  pub PartitionSize: UINT64,
  ///
  /// Signature unique to this partition:
  /// If SignatureType is 0, this field has to be initialized with 16 zeros.
  /// If SignatureType is 1, the MBR signature is stored in the first 4 bytes of this field.
  /// The other 12 bytes are initialized with zeros.
  /// If SignatureType is 2, this field contains a 16 byte signature.
  ///
  pub Signature: [UINT8;16],
  ///
  /// Partition Format: (Unused values reserved).
  /// 0x01 - PC-AT compatible legacy MBR.
  /// 0x02 - GUID Partition Table.
  ///
  pub MBRType: UINT8,
  ///
  /// Type of Disk Signature: (Unused values reserved).
  /// 0x00 - No Disk Signature.
  /// 0x01 - 32-bit signature from address 0x1b8 of the type 0x01 MBR.
  /// 0x02 - GUID signature.
  ///
  pub SignatureType: UINT8,
}

pub const MBR_TYPE_PCAT: UINTN = 0x01;
pub const MBR_TYPE_EFI_PARTITION_TABLE_HEADER: UINTN = 0x02;

pub const NO_DISK_SIGNATURE: UINTN = 0x00;
pub const SIGNATURE_TYPE_MBR: UINTN = 0x01;
pub const SIGNATURE_TYPE_GUID: UINTN = 0x02;

///
/// CD-ROM Media Device Path SubType.
///
pub const MEDIA_CDROM_DP: UINT8 = 0x02;

///
/// The CD-ROM Media Device Path is used to define a system partition that exists on a CD-ROM.
///
#[repr(packed)]
pub struct CDROM_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  ///
  /// Boot Entry number from the Boot Catalog. The Initial/Default entry is defined as zero.
  ///
  pub BootEntry: UINT32,
  ///
  /// Starting RBA of the partition on the medium. CD-ROMs use Relative logical Block Addressing.
  ///
  pub PartitionStart: UINT64,
  ///
  /// Size of the partition in units of Blocks, also called Sectors.
  ///
  pub PartitionSize: UINT64,
}

//
// Use VENDOR_DEVICE_PATH struct
//
pub const MEDIA_VENDOR_DP: UINT8 = 0x03;  //< Media vendor device path subtype.;

///
/// File Path Media Device Path SubType
///
pub const MEDIA_FILEPATH_DP: UINT8 = 0x04;

#[repr(packed)]
pub struct FILEPATH_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  ///
  /// A NULL-terminated Path string including directory and file names.
  ///
  pub PathName: [CHAR16;1],
}

// #define SIZE_OF_FILEPATH_DEVICE_PATH  OFFSET_OF(FILEPATH_DEVICE_PATH,PathName)

///
/// Media Protocol Device Path SubType.
///
pub const MEDIA_PROTOCOL_DP: UINT8 = 0x05;

///
/// The Media Protocol Device Path is used to denote the protocol that is being 
/// used in a device path at the location of the path specified. 
/// Many protocols are inherent to the style of device path.
///
#[repr(packed)]
pub struct MEDIA_PROTOCOL_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  ///
  /// The ID of the protocol.
  ///
  pub Protocol: EFI_GUID,
}

///
/// PIWG Firmware File SubType.
///
pub const MEDIA_PIWG_FW_FILE_DP: UINT8 = 0x06;

///
/// This device path is used by systems implementing the UEFI PI Specification 1.0 to describe a firmware file.
///
#[repr(packed)]
pub struct MEDIA_FW_VOL_FILEPATH_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  ///
  /// Firmware file name
  ///
  pub FvFileName: EFI_GUID,
}

///
/// PIWG Firmware Volume Device Path SubType.
///
pub const MEDIA_PIWG_FW_VOL_DP: UINT8 = 0x07;

///
/// This device path is used by systems implementing the UEFI PI Specification 1.0 to describe a firmware volume.
///
#[repr(packed)]
pub struct MEDIA_FW_VOL_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  ///
  /// Firmware volume name.
  ///
  pub FvName: EFI_GUID,
}

///
/// Media relative offset range device path.
///
pub const MEDIA_RELATIVE_OFFSET_RANGE_DP: UINT8 = 0x08;

///
/// Used to describe the offset range of media relative.
///
#[repr(packed)]
pub struct MEDIA_RELATIVE_OFFSET_RANGE_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  pub Reserved: UINT32,
  pub StartingOffset: UINT64,
  pub EndingOffset: UINT64,
}

///
/// BIOS Boot Specification Device Path.
///
pub const BBS_DEVICE_PATH: UINT8 = 0x05;

///
/// BIOS Boot Specification Device Path SubType.
///
pub const BBS_BBS_DP: UINT8 = 0x01;

///
/// This Device Path is used to describe the booting of non-EFI-aware operating systems.
///
pub struct BBS_BBS_DEVICE_PATH {
  pub Header: EFI_DEVICE_PATH_PROTOCOL,
  ///
  /// Device Type as defined by the BIOS Boot Specification.
  ///
  pub DeviceType: UINT16,
  ///
  /// Status Flags as defined by the BIOS Boot Specification.
  ///
  pub StatusFlag: UINT16,
  ///
  /// Null-terminated ASCII string that describes the boot device to a user.
  ///
  pub String: [CHAR8;1],
}

//
// DeviceType definitions - from BBS specification
//
pub const BBS_TYPE_FLOPPY: UINTN = 0x01;
pub const BBS_TYPE_HARDDRIVE: UINTN = 0x02;
pub const BBS_TYPE_CDROM: UINTN = 0x03;
pub const BBS_TYPE_PCMCIA: UINTN = 0x04;
pub const BBS_TYPE_USB: UINTN = 0x05;
pub const BBS_TYPE_EMBEDDED_NETWORK: UINTN = 0x06;
pub const BBS_TYPE_BEV: UINTN = 0x80;
pub const BBS_TYPE_UNKNOWN: UINTN = 0xFF;


pub const END_DEVICE_PATH_TYPE: UINT8 = 0x7f;
pub const END_ENTIRE_DEVICE_PATH_SUBTYPE: UINT8 = 0xFF;
pub const END_INSTANCE_DEVICE_PATH_SUBTYPE: UINT8 = 0x01;




// COPIED from tianocore

// THE FOLLOWING NOT IMPLEMENTED YET
// #define PNP_EISA_ID_CONST         0x41d0
// #define EISA_ID(_Name, _Num)      ((UINT32)((_Name) | (_Num) << 16))
// #define EISA_PNP_ID(_PNPId)       (EISA_ID(PNP_EISA_ID_CONST, (_PNPId)))
// #define EFI_PNP_ID(_PNPId)        (EISA_ID(PNP_EISA_ID_CONST, (_PNPId)))

// #define PNP_EISA_ID_MASK          0xffff
// #define EISA_ID_TO_NUM(_Id)       ((_Id) >> 16)

// #define ACPI_ADR_DP               0x03

// ///
// /// The _ADR device path is used to contain video output device attributes to support the Graphics
// /// Output Protocol. The device path can contain multiple _ADR entries if multiple video output
// /// devices are displaying the same output.
// ///
// typedef struct {
//   EFI_DEVICE_PATH_PROTOCOL        Header;
//   ///
//   /// _ADR value. For video output devices the value of this
//   /// field comes from Table B-2 of the ACPI 3.0 specification. At
//   /// least one _ADR value is required.
//   ///
//   UINT32                          ADR;
//   //
//   // This device path may optionally contain more than one _ADR entry.
//   //
// } ACPI_ADR_DEVICE_PATH;

// #define ACPI_ADR_DISPLAY_TYPE_OTHER             0
// #define ACPI_ADR_DISPLAY_TYPE_VGA               1
// #define ACPI_ADR_DISPLAY_TYPE_TV                2
// #define ACPI_ADR_DISPLAY_TYPE_EXTERNAL_DIGITAL  3
// #define ACPI_ADR_DISPLAY_TYPE_INTERNAL_DIGITAL  4

// #define ACPI_DISPLAY_ADR(_DeviceIdScheme, _HeadId, _NonVgaOutput, _BiosCanDetect, _VendorInfo, _Type, _Port, _Index) \
//           ((UINT32)( (((_DeviceIdScheme) & 0x1) << 31) |  \
//                       (((_HeadId)         & 0x7) << 18) |  \
//                       (((_NonVgaOutput)   & 0x1) << 17) |  \
//                       (((_BiosCanDetect)  & 0x1) << 16) |  \
//                       (((_VendorInfo)     & 0xf) << 12) |  \
//                       (((_Type)           & 0xf) << 8)  |  \
//                       (((_Port)           & 0xf) << 4)  |  \
//                        ((_Index)          & 0xf) ))
