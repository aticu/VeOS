//!Deals with the page tables.
mod page_table;
mod page_table_entry;

pub fn debug() {
    let current_page_table = unsafe { page_table::CurrentPageTable::new() };
}

///Tests for methods used in the paging module.
#[cfg(test)]
mod tests {
    use super::*;

    ///Tests the points_to method of a page table entry.
    #[test]
    fn test_points_to() {
        let mut entry = PageTableEntry::new();
        entry.set_address(0xdeadb000);
        assert_eq!(entry.points_to(), None);
        entry.set_flags(PRESENT);
        assert_eq!(entry.points_to(), Some(0xdeadb000));
    }

    ///Tests that unaligned addresses panic.
    #[test]
    #[should_panic]
    fn test_unaligned_address() {
        let mut entry = PageTableEntry::new();
        entry.set_address(0xdeadbeef);
    }

    ///Tests that overflowing addresses panic.
    #[test]
    #[should_panic]
    fn test_address_overflow() {
        let mut entry = PageTableEntry::new();
        entry.set_address(0xcafebabedeadb000);
    }

    ///Tests that the flags field works as expected.
    #[test]
    fn test_flags() {
        let mut entry = PageTableEntry::new();
        let flags = PRESENT | DIRTY | USER_ACCESSIBLE | WRITABLE | NO_EXECUTE;
        entry.set_flags(flags);
        assert_eq!(entry.flags(), flags);
    }

    ///Tests that the binary representation is as expected.
    #[test]
    fn test_representation() {
        let mut entry = PageTableEntry::new();
        let flags = PRESENT | DIRTY | USER_ACCESSIBLE | WRITABLE | NO_EXECUTE;
        entry.set_flags(flags);
        entry.set_address(0xdeadb000);
        assert_eq!(entry.0, 0xdeadb000 | (1 << 0) | (1 << 6) | (1 << 2) | (1 << 1) | (1 << 63));
    }
}
