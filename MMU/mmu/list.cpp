#include"list.h"
LINKNODE LRU_list;

//链表初始化
void pInitList(LINKNODE &pHead)
{
	pHead = (LINKNODE)malloc(sizeof(NODE));
	pHead->next = NULL;
};

//节点位置从1开始，头节点位置为0
int pInsertElem(LINKNODE &pHead, LINKNODE s, int posi)
{
	LINKNODE p;
	int counter = 0;
	if (pHead == NULL)
	{
		return -1;
	}
	p = pHead;
	while (p != NULL)
	{
		if (counter == posi - 1)
		{
			s->next = p->next;
			p->next = s;
			return 1;
		}
		p = p->next;
		counter++;
	}
	return -1;
};

//从将元素从当前位置移动到第一个位置
int pMovetoFirst(LINKNODE &pHead, int e)
{
	LINKNODE p = pHead;
	LINKNODE q;
	if (pHead == NULL) return -1;
	while ((p != NULL) && (p->next->frame_number != e)) p = p->next;
	if (p == NULL) return -1;//无元素e
	q = p->next;
	p->next = q->next;
	q->next = pHead->next;
	pHead->next = q;
	return 1;
}

//得到末尾节点的指针
LINKNODE GetEndNode(LINKNODE pHead)
{
	LINKNODE p = pHead;
	if (pHead == NULL) return NULL;
	while (p->next != NULL) p = p->next;
	return p;
}

int pTraverseList(LINKNODE pHead)
{
	LINKNODE p;
	if (pHead == NULL)
	{
		return -1;
	}
	p = pHead->next;
	while (p != NULL)
	{
		printf("%d ", p->frame_number);
		p = p->next;
	}
	return 1;
}
