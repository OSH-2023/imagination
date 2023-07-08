#include"list.h"
LINKNODE LRU_list;

//�����ʼ��
void pInitList(LINKNODE &pHead)
{
	pHead = (LINKNODE)malloc(sizeof(NODE));
	pHead->next = NULL;
};

//�ڵ�λ�ô�1��ʼ��ͷ�ڵ�λ��Ϊ0
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

//�ӽ�Ԫ�شӵ�ǰλ���ƶ�����һ��λ��
int pMovetoFirst(LINKNODE &pHead, int e)
{
	LINKNODE p = pHead;
	LINKNODE q;
	if (pHead == NULL) return -1;
	while ((p != NULL) && (p->next->frame_number != e)) p = p->next;
	if (p == NULL) return -1;//��Ԫ��e
	q = p->next;
	p->next = q->next;
	q->next = pHead->next;
	pHead->next = q;
	return 1;
}

//�õ�ĩβ�ڵ��ָ��
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
