using System.Collections.Generic;
using Games.Cheetah.Client.ServerAPI.Mock.Type;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI.Mock.Storage
{
    public abstract class AbstractStorage<T>
    {
        protected readonly Dictionary<FieldKey, T> fields = new();

        public byte Set(ushort clientId, in CheetahObjectId objectId, ushort fieldId, ref T data)
        {
            var key = new FieldKey(objectId, fieldId);
            fields[key] = data;
            return 0;
        }

        public bool TryGetFieldValue(CheetahObjectId objectId, ushort fieldId, out T value)
        {
            var key = new FieldKey(objectId, fieldId);
            return fields.TryGetValue(key, out value);
        }


        public void Clear()
        {
            fields.Clear();
        }

        public void DeleteField(in CheetahObjectId objectId, ushort fieldId)
        {
            fields.Remove(new FieldKey(objectId, fieldId));
        }
    }
}