using Games.Cheetah.Client.Internal.FFI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI.FFI
{
    public class LongServerAPI : ILongServerAPI
    {
        public byte SetListener(ushort clientId, ILongServerAPI.Listener listener)
        {
            return LongFFI.SetListener(clientId, listener);
        }


        public byte Set(ushort clientId, in CheetahObjectId objectId, FieldId.Long fieldId, long value)
        {
            return LongFFI.Set(clientId, in objectId, fieldId.Id, value);
        }


        public byte Increment(ushort clientId, in CheetahObjectId objectId, FieldId.Long fieldId, long increment)
        {
            return LongFFI.Increment(clientId, in objectId, fieldId.Id, increment);
        }


        public byte CompareAndSet(ushort clientId, in CheetahObjectId objectId, FieldId.Long fieldId, long currentValue, long newValue,
            bool hasReset, long resetValue)
        {
            return LongFFI.CompareAndSet(clientId, in objectId, fieldId.Id, currentValue, newValue, hasReset, resetValue);
        }
    }
}