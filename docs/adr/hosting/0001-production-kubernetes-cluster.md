# Устройство kubernetes кластера для production

## Статус

## Контекст и постановка проблемы

Необходимо обеспечить изолированность сервисов по CPU/Memory, плюс дать возможность
быстрого добавления ресурсов для конкретного типа сервисов.

## Решение
### Разделяем сервисы по нодам
- relay сервисы вынесены на отдельные node
- один relay сервис на одной node
- мониторинг вынесен на отдельные node
- nginx вынесен на отдельную node
- все остальные cheetah сервисы - на отдельной node
- Итого 4 типа нод

Используем nodeSelector + Tolerations для распределения сервисов по серверам
Формат: 
    type = relay|monitoring|others|nginx