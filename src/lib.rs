#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

type NTSTATUS = i32;
type PVOID = *mut core::ffi::c_void;

const STATUS_SUCCESS: NTSTATUS = 0;



// FLT_REGISTRATION = https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/fltkernel/ns-fltkernel-_flt_registration

#[repr(C)]
struct FLT_REGISTRATION {
    size: u16,
    version: u16,
    flags: u32,
    context_registration: PVOID,
    operation_callbacks: *const FLT_OPERATION_REGISTRATION,
    filter_unload_callback: PVOID,
    instance_setup_callback: PVOID,
    instance_query_teardown_callback: PVOID,
    instance_teardown_start_callback: PVOID,
    instance_teardown_complete_callback: PVOID,
    generate_file_name_callback: PVOID,
    normalize_name_component_callback: PVOID,
    normalize_context_cleanup_callback: PVOID,
    transaction_notification_callback: PVOID,
    normalize_name_component_ex_callback: PVOID,
    section_notification_callback: PVOID,
}

// FLT_OPERATION_REGISTRATION = https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/fltkernel/ns-fltkernel-_flt_operation_registration

#[repr(C)]
struct FLT_OPERATION_REGISTRATION {
    major_function: u8,
    flags: u32,
    pre_operation: PVOID,
    post_operation: PVOID,
    reserved1: PVOID,
}

const IRP_MJ_WRITE: u8 = 0x04; // wdm.h
const IRP_MJ_OPERATION_END: u8 = 0x80; // fltkernel
const FLT_REGISTRATION_VERSION: u16 = 0x0203; // win10+

const FLT_PREOP_SUCCESS_NO_CALLBACK: u32 = 1;

// external kernel API

#[link(name = "fltmgr")]
extern "system" {
    fn FltRegisterFilter(
        driver: PVOID,
        registration: *const FLT_REGISTRATION,
        ret_filter: *mut PVOID,
    ) -> NTSTATUS;

    fn FltStartFiltering(filter: PVOID) -> NTSTATUS;
}

#[link(name = "ntoskrnl")]
extern "system" {
    fn DbgPrint(msg: *const u8, ...) -> i32;
}

// WRITE CALLBACK
unsafe extern "system" fn pre_write_callback(
    data: PVOID,
    _flt_objects: PVOID,
    _completion_context: PVOID,
) -> u32 {
    let mut name_info: *mut FLT_FILE_NAME_INFORMATION = core::ptr::null_mut();

    // Get file information
    let status = FltGetFileNameInformation(
        data,
        FLT_FILE_NAME_NORMALIZED | FLT_FILE_NAME_QUERY_DEFAULT,
        &mut name_info,
    );

    if status == STATUS_SUCCESS {
        // Parse the name information (https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/fltkernel/nf-fltkernel-fltparsefilenameinformation)
        FltParseFileNameInformation(name_info);
        
        let msg = b"new file on disk detected: %wZ\n\0";
        DbgPrint(msg.as_ptr(), &(*name_info).Name);

        // relese memory
        FltReleaseFileNameInformation(name_info);
    } else {
        let err_msg = b"error: \n\0";
        DbgPrint(err_msg.as_ptr());
    }

    FLT_PREOP_SUCCESS_NO_CALLBACK
}

// FLT_OPERATION_REGISTRATION - if its 0x80 stop reading memory (value of IRP_MJ_OPERATION_END)
unsafe impl Sync for FLT_OPERATION_REGISTRATION {}
static OPERATION_REGISTRATION: [FLT_OPERATION_REGISTRATION; 2] = [
    FLT_OPERATION_REGISTRATION {
        major_function: IRP_MJ_WRITE,
        flags: 0,
        pre_operation: pre_write_callback as PVOID,
        post_operation: core::ptr::null_mut(),
        reserved1: core::ptr::null_mut(),
    },
    // Terminator
    FLT_OPERATION_REGISTRATION {
        major_function: IRP_MJ_OPERATION_END,
        flags: 0,
        pre_operation: core::ptr::null_mut(),
        post_operation: core::ptr::null_mut(),
        reserved1: core::ptr::null_mut(),
    },
];


// impl of FLT_REGISTRATION like in winapi
unsafe impl Sync for FLT_REGISTRATION {}

static REGISTRATION: FLT_REGISTRATION = FLT_REGISTRATION {
    size: core::mem::size_of::<FLT_REGISTRATION>() as u16,
    version: FLT_REGISTRATION_VERSION,
    flags: 0,
    context_registration: core::ptr::null_mut(),
    operation_callbacks: OPERATION_REGISTRATION.as_ptr(),
    filter_unload_callback: core::ptr::null_mut(),
    instance_setup_callback: core::ptr::null_mut(),
    instance_query_teardown_callback: core::ptr::null_mut(),
    instance_teardown_start_callback: core::ptr::null_mut(),
    instance_teardown_complete_callback: core::ptr::null_mut(),
    generate_file_name_callback: core::ptr::null_mut(),
    normalize_name_component_callback: core::ptr::null_mut(),
    normalize_context_cleanup_callback: core::ptr::null_mut(),
    transaction_notification_callback: core::ptr::null_mut(),
    normalize_name_component_ex_callback: core::ptr::null_mut(),
    section_notification_callback: core::ptr::null_mut(),
};

// i need to implement unicode_string as it is in winapi
// because of no_std i can't handle text
#[repr(C)]
pub struct UNICODE_STRING {
    pub Length: u16,
    pub MaximumLength: u16,
    pub Buffer: *mut u16,
}

// required by msvc linker for compiling
#[no_mangle]
pub static _fltused: i32 = 0;

#[no_mangle]
pub extern "system" fn __CxxFrameHandler3() -> i32 {
    0
}

// struct for parsed components of a filepath
#[repr(C)]
pub struct FLT_FILE_NAME_INFORMATION {
    pub Size: u16,
    pub NamesParsed: u16,
    pub Format: u32,
    pub Name: UNICODE_STRING,
    pub Volume: UNICODE_STRING,
    pub Share: UNICODE_STRING,
    pub Extension: UNICODE_STRING,
    pub Stream: UNICODE_STRING,
    pub FinalComponent: UNICODE_STRING,
    pub ParentDir: UNICODE_STRING,
}

// Flags for FltGetFileNameInformation
const FLT_FILE_NAME_NORMALIZED: u32 = 0x01;
const FLT_FILE_NAME_QUERY_DEFAULT: u32 = 0x0100;

#[link(name = "fltmgr")]
extern "system" {
    fn FltGetFileNameInformation(
        callback_data: PVOID,
        name_options: u32,
        name_information: *mut *mut FLT_FILE_NAME_INFORMATION,
    ) -> NTSTATUS;

    fn FltParseFileNameInformation(
        name_information: *mut FLT_FILE_NAME_INFORMATION,
    ) -> NTSTATUS;

    fn FltReleaseFileNameInformation(
        name_information: *mut FLT_FILE_NAME_INFORMATION,
    );
}

// Driver entry point
#[no_mangle]
pub extern "system" fn DriverEntry(
    driver_object: PVOID,
    _registry_path: PVOID,
) -> NTSTATUS {
    let mut filter: PVOID = core::ptr::null_mut();

    unsafe {
        let status = FltRegisterFilter(
            driver_object,
            &REGISTRATION,
            &mut filter,
        );

        if status != STATUS_SUCCESS {
            return status;
        }

        // begin intercepting I/O
        FltStartFiltering(filter);
    }

    STATUS_SUCCESS
}