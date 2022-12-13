using Games.Cheetah.Client.Internal.FFI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI.FFI
{
    public class DoubleServerAPI : IDoubleServerAPI
    {
        public byte SetListener(ushort clientId, IDoubleServerAPI.Listener listener)
        {
            return DoubleFFI.SetListener(clientId, listener);
        }

        public byte Set(ushort clientId, in CheetahObjectId objectId, ushort fieldId, double value)
        {
            return DoubleFFI.Set(clientId, objectId, fieldId, value);
        }

        public byte Increment(ushort clientId, in CheetahObjectId objectId, ushort fieldId, double increment)
        {
            return DoubleFFI.Increment(clientId, in objectId, fieldId, increment);
        }
    }
}