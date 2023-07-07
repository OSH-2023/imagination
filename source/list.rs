/// 
/// pub type ListItem_t = xLIST_ITEM;
/// pub type ItemLink = Arc<RwLock<ListItem_t>>;
/// pub type WeakItemLink = Weak<RwLock<ListItem_t>>;
/// 
/// pub type List_t = xLIST;
/// pub type ListLink = Arc<RwLock<List_t>>;
/// pub type WeakListLink = Weak<RwLock<List_t>>;


use std::sync::{Arc, RwLock, Weak};

use crate::port::{portMAX_DELAY, TickType_t, UBaseType_t};
use crate::task::{TCB, TaskHandle};

/// 暂时未实现：listGET_NEXT listGET_END_MARKER
/// not varified: TCB、 TaskHandle

pub struct xLIST_ITEM
{
    // 辅助值，用于帮助结点做顺序排列
    /*< The value being listed.  In most cases this is used to sort the list in descending order. */
	xItemValue: TickType_t,	
    // 双向引用
    /*< Pointer to the next ListItem_t in the list. */	
	pxNext: WeakItemLink,     
    // 双向引用
    /*< Pointer to the previous ListItem_t in the list. */	
	pxPrevious: WeakItemLink,	
	// 指向拥有该结点的内核对象，通常是TCB
    /*< Pointer to the object (normally a TCB) that contains the list item.  
    There is therefore a two way link between the object containing the list 
    item and the list item itself. */
    pvOwner: Weak<RwLock<TCB>>,										
	// 指向该节点所在的链表 双向引用
    /*< Pointer to the list in which this list item is placed (if any). */
    pvContainer: Weak<RwLock<List_t>>,				
}
pub type ListItem_t = xLIST_ITEM;	

pub type ItemLink = Arc<RwLock<ListItem_t>>;
pub type WeakItemLink = Weak<RwLock<ListItem_t>>;

impl Default for xLIST_ITEM{
    // 替代 vListInitialiseItem
    fn default() -> Self {
        xLIST_ITEM{
            /* The list end value is the highest possible value in the list to
            ensure it remains at the end of the list. */
            xItemValue: portMAX_DELAY,
            pxNext: Default::default(),
            pxPrevious: Default::default(),
            pvOwner: Default::default(),
            /* Make sure the list item is not recorded as being on a list. */
            pvContainer: Default::default(),
        }
    }
}

impl xLIST_ITEM {
    // item_value 
    //pub fn set_value(mut self, item_value: TickType_t) -> Self {
    //    self.xItemValue = item_value;
    //    self
    //}

    pub fn new(item_value: TickType_t) -> Self {
        let mut item = ListItem_t::default();
        item.set_value(item_value);
        item
    }

    pub fn set_value(&mut self, item_value: TickType_t) {
        self.xItemValue = item_value;
    }

    pub fn owner(mut self, owner: TaskHandle) -> Self {
        self.pvOwner = owner.into();
        self
    }

    pub fn set_container(&mut self, container: &Arc<RwLock<List_t>>) {
        self.pvContainer = Arc::downgrade(container);
    }

    
    fn remove(&mut self, link: WeakItemLink) -> UBaseType_t {
        /* The list item knows which list it is in.  Obtain the list from the list
        item. */
        let list = self
            .pvContainer
            .upgrade()
            .unwrap_or_else(|| panic!("Container not set"));
        let ret_val = list
            .write()
            .unwrap()
            .remove_item(&self, link);
        set_list_item_next(&self.pxPrevious, Weak::clone(&self.pxNext));
        set_list_item_prev(&self.pxNext, Weak::clone(&self.pxPrevious));
        self.pvContainer = Weak::new();
        ret_val
    }
    
}

/// get Arc<RwLock<xLIST_ITEM>>
pub fn new_list_item(item_value: TickType_t) -> Arc<RwLock<xLIST_ITEM>> {
    let mut raw_list_item = ListItem_t::default();
    raw_list_item.set_value(item_value);
    let item: Arc<RwLock<xLIST_ITEM>> = Arc::new(RwLock::new(raw_list_item));
    item
}

/* struct xMINI_LIST_ITEM
{
    xItemValue: TickType_t,	
    /*< Pointer to the next ListItem_t in the list. */	
	pxNext: WeakItemLink,     
    /*< Pointer to the previous ListItem_t in the list. */	
	pxPrevious: WeakItemLink,
}
type MiniListItem_t = xMINI_LIST_ITEM;
pub type MiniItemLink = Arc<RwLock<MiniListItem_t>>;
pub type WeakMiniItemLink = Weak<RwLock<MiniListItem_t>>; */

pub struct xLIST
{
    // 链表节点计数器
    uxNumberOfItems: UBaseType_t,
    // 链表节点索引指针
    /*< Used to walk through the list.  Points to the last item returned by a call 
    to listGET_OWNER_OF_NEXT_ENTRY (). */
	pxIndex: WeakItemLink,			
	// 链表最后一个节点 单向引用
    /*< List item that contains the maximum possible item value meaning it is always 
    at the end of the list and is therefore used as a marker. */
    xListEnd: ItemLink ,							
}
pub type List_t = xLIST;
pub type ListLink = Arc<RwLock<List_t>>;
pub type WeakListLink = Weak<RwLock<List_t>>;

impl Default for xLIST{
    // 替代 vListInitialise
    fn default() -> Self {
        /* The list structure contains a list item which is used to mark the
        end of the list.  To initialise the list the list end is inserted
        as the only list entry. */
        // 链表的最后节点（节点的辅助排序值设为 portMAX_DELAY 最大
        let list_end: ItemLink = Arc::new(RwLock::new(ListItem_t::default()));

        // 将最后一个节点的 pxNext 与 pxPrevious 指向自身
        /* The list end next and previous pointers point to itself so we know
        when the list is empty. */
        (*list_end.write().unwrap()).pxNext = Arc::downgrade(&list_end);
        (*list_end.write().unwrap()).pxPrevious = Arc::downgrade(&list_end);

        xLIST {
            // 计数器为0，链表为空
            uxNumberOfItems: 0,
            // 索引指向最后一个节点
            pxIndex: Arc::downgrade(&list_end),
            xListEnd: list_end,
        }
    }
}

impl xLIST {
    pub fn traverse(& self) {
        let mut iterator = Arc::downgrade(&self.xListEnd);
        loop {
            /* There is nothing to do here, just iterating to the wanted
            insertion position. */
            let next = get_list_item_next(&iterator);
            let value = get_weak_item_value(&next);
            println!("value: {}", value);
            if value == portMAX_DELAY {
                break;
            }
            iterator = next;
        }
    }

    fn insert(&mut self, item_link: WeakItemLink) {
        println!("in");
        let value_of_insertion = get_weak_item_value(&item_link);
        /* Insert the new list item into the list, sorted in xItemValue order.

        If the list already contains a list item with the same item value then the
        new list item should be placed after it.  This ensures that TCB's which are
        stored in ready lists (all of which have the same xItemValue value) get a
        share of the CPU.  However, if the xItemValue is the same as the back marker
        the iteration loop below will not end.  Therefore the value is checked
        first, and the algorithm slightly modified if necessary. */
        let item_to_insert = if value_of_insertion == portMAX_DELAY {
            get_list_item_prev(&Arc::downgrade(&self.xListEnd))
        } else {
            let mut iterator = Arc::downgrade(&self.xListEnd);
            loop {
                /* There is nothing to do here, just iterating to the wanted
                insertion position. */
                let next = get_list_item_next(&iterator);
                if get_weak_item_value(&next) > value_of_insertion {
                    break iterator;
                }
                iterator = next;
            }
        };

        let prev = Weak::clone(&item_to_insert);
        let next = get_list_item_next(&item_to_insert);

        set_list_item_next(&item_link, Weak::clone(&next));
        set_list_item_prev(&item_link, Weak::clone(&prev));
        set_list_item_next(&prev, Weak::clone(&item_link));
        set_list_item_prev(&next, Weak::clone(&item_link));

        self.uxNumberOfItems += 1;
    }

    // insert before pxIndex
    pub fn insert_end(&mut self, item_link: WeakItemLink) {
        let prev = get_list_item_prev(&self.pxIndex);
        let next = Weak::clone(&self.pxIndex);
        set_list_item_next(&item_link, Weak::clone(&next));
        set_list_item_prev(&item_link, Weak::clone(&prev));
        set_list_item_next(&prev, Weak::clone(&item_link));
        set_list_item_prev(&next, Weak::clone(&item_link));

        self.uxNumberOfItems += 1;
    }

    fn remove_item(&mut self, item: &xLIST_ITEM, link: WeakItemLink) -> UBaseType_t {
        // TODO: Find a more effiecient
        if Weak::ptr_eq(&link, &self.pxIndex) {
            self.pxIndex = Weak::clone(&item.pxPrevious);
        }

        self.uxNumberOfItems -= 1;

        self.uxNumberOfItems
    }

    fn is_empty(&self) -> bool {
        self.uxNumberOfItems == 0
    }

    fn get_length(&self) -> UBaseType_t {
        self.uxNumberOfItems
    }

    pub fn increment_index(&mut self) {
        self.pxIndex = get_list_item_next(&self.pxIndex);
        if Weak::ptr_eq(&self.pxIndex, &Arc::downgrade(&self.xListEnd)) {
            self.pxIndex = get_list_item_next(&self.pxIndex);
        }
    }

    
    fn get_owner_of_next_entry(&mut self) -> Weak<RwLock<TCB>> {
        self.increment_index();
        let owned_index = self
            .pxIndex
            .upgrade()
            .unwrap_or_else(|| panic!("List item is None"));
        let owner = Weak::clone(&owned_index.read().unwrap().owner);
        owner
    }

    fn get_owner_of_head_entry(&self) -> Weak<RwLock<TCB>> {
        let list_end = get_list_item_next(&Arc::downgrade(&self.xListEnd));
        let owned_index = list_end
            .upgrade()
            .unwrap_or_else(|| panic!("List item is None"));
        let owner = Weak::clone(&owned_index.read().unwrap().owner);
        owner
    }
    
}

// 替换 item.pxNext = next;
fn set_list_item_next(item: &WeakItemLink, next: WeakItemLink) {
    let owned_item: Arc<RwLock<xLIST_ITEM>> = item
        .upgrade()
        .unwrap_or_else(|| panic!("List item is None"));
    // owned_item: Arc<RwLock<xLIST_ITEM>>
    // owned_item.write(): Result<RwLockWriteGuard<xLIST_ITEM>, PoisonError<RwLockWriteGuard<xLIST_ITEM>>>
    // owned_item.write().unwrap(): RwLockWriteGuard<xLIST_ITEM>
    // *owned_item.write().unwrap(): xLIST_ITEM
    (*owned_item.write().unwrap()).pxNext = next;
}

// 替换 item.pxPrevious = prev;
fn set_list_item_prev(item: &WeakItemLink, prev: WeakItemLink) {
    let owned_item = item
        .upgrade()
        .unwrap_or_else(|| panic!("List item is None"));
    (*owned_item.write().unwrap()).pxPrevious = prev;
}

fn get_list_item_next(item: &WeakItemLink) -> WeakItemLink {
    let owned_item = item
        .upgrade()
        .unwrap_or_else(|| panic!("List item is None"));
    let next = Weak::clone(&(*owned_item.read().unwrap()).pxNext);
    next
}

fn get_list_item_prev(item: &WeakItemLink) -> WeakItemLink {
    let owned_item: Arc<RwLock<xLIST_ITEM>> = item
        .upgrade()
        .unwrap_or_else(|| panic!("List item is None"));
    let prev = Weak::clone(&(*owned_item.read().unwrap()).pxPrevious);
    prev
}

/// get_list_item_value
pub fn listGET_LIST_ITEM_VALUE(item: &ItemLink) -> TickType_t {
    (*item.read().unwrap()).xItemValue
}

/// set_list_item_value
pub fn listSET_LIST_ITEM_VALUE(item: &ItemLink, item_value: TickType_t) {
    (*item.write().unwrap()).xItemValue = item_value;
}

pub fn get_weak_item_value(item: &WeakItemLink) -> TickType_t {
    let owned_item: Arc<RwLock<xLIST_ITEM>> = item
        .upgrade()
        .unwrap_or_else(|| panic!("List item is None"));
    let value = (*owned_item.read().unwrap()).xItemValue;
    value
}

pub fn set_weak_item_value(item: &WeakItemLink, item_value: TickType_t) {
    let owned_item = item
        .upgrade()
        .unwrap_or_else(|| panic!("List item is None"));
    owned_item.write().unwrap().xItemValue = item_value;
}

pub fn get_list_item_container(item: &ItemLink) -> Option<ListLink> {
    //let owned_item = item.upgrade().unwrap_or_else(|| panic!("List item is None"));
    let container = Weak::clone(&(*item.read().unwrap()).pvContainer);
    container.upgrade()
}

pub fn vListInitialise(item: &mut ItemLink) {
    let ItemLink = xLIST::default();
}

/// 获取链表的入口节点
/// return type: Weak<RwLock<xLIST_ITEM>>
pub fn listGET_HEAD_ENTRY( list: &ListLink ) -> Weak<RwLock<xLIST_ITEM>>{
    let list_end = Arc::downgrade(&list.read().unwrap().xListEnd);
    get_list_item_next(&list_end)
}

/// 获取链表根节点的节点计数器的值
pub fn listGET_ITEM_VALUE_OF_HEAD_ENTRY(list: &ListLink) -> TickType_t{
    let list_end = Arc::downgrade(&list.read().unwrap().xListEnd);
    let list_head = get_list_item_next(&list_end);
    get_weak_item_value(&list_head)
}

/// list_is_empty
pub fn listLIST_IS_EMPTY(list: &ListLink) -> bool {
    list.read().unwrap().is_empty()
}

/// current_list_length
pub fn listCURRENT_LIST_LENGTH(list: &ListLink) -> UBaseType_t {
    list.read().unwrap().get_length()
}


pub fn get_list_item_owner(item_link: &ItemLink) -> TaskHandle {
    let owner = Weak::clone(&item_link.read().unwrap().pvOwner);
    owner.into()
}

pub fn set_list_item_owner(item_link: &ItemLink, owner: TaskHandle) {
    item_link.write().unwrap().pvOwner = owner.into()
} 

pub fn get_owner_of_next_entry(list: &ListLink) -> TaskHandle {
    let task = list.write().unwrap().get_owner_of_next_entry();
    task.into()
}

pub fn get_owner_of_head_entry(list: &ListLink) -> TaskHandle {
    let task = list.read().unwrap().get_owner_of_head_entry();
    task.into()
}


pub fn is_contained_within(list: &ListLink, item_link: &ItemLink) -> bool {
    match get_list_item_container(&item_link) {
        Some(container) => Arc::ptr_eq(list, &container),
        None => false,
    }
}

// why not use &ItemLink
/// list_insert
/// 将节点按照升序排列插入到链表
pub fn vListInsert(list: &ListLink, item_link: &ItemLink) {
    /* Remember which list the item is in.  This allows fast removal of the
    item later. */
    item_link.write().unwrap().set_container(&list);
    println!("Set conatiner");
    list.write().unwrap().insert(Arc::downgrade(&item_link))
}

// why not use &ItemLink
/// list_insert_end
/// insert before pxIndex
pub fn vListInsertEnd(list: &ListLink, item_link: &ItemLink) {
    /* Insert a new list item into pxList, but rather than sort the list,
    makes the new list item the last item to be removed by a call to
    listGET_OWNER_OF_NEXT_ENTRY(). */

    /* Remember which list the item is in. */
    item_link.write().unwrap().set_container(&list);

    list.write().unwrap().insert_end(Arc::downgrade(&item_link))
}

// why not use &ItemLink
/// list_remove
/// 传回剩余节点数
pub fn uxListRemove(item_link: ItemLink) -> UBaseType_t {
    item_link
        .write()
        .unwrap()
        .remove(Arc::downgrade(&item_link))
}


