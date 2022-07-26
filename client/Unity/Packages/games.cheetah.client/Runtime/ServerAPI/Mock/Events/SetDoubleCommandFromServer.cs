using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI.Mock.Events
{
    public class SetDoubleCommandFromServer : ICommandFromServer
    {
        private readonly CheetahObjectId objectId;
        private readonly FieldId.Double fieldId;
        private readonly double value;

        public SetDoubleCommandFromServer(CheetahObjectId objectId, FieldId.Double fieldId, double value)
        {
            this.objectId = objectId;
            this.fieldId = fieldId;
            this.value = value;
        }

        public void Apply(CheetahClientMock cheetahClientMock)
        {
            cheetahClientMock.doubles.listener?.Invoke(0, in objectId, fieldId.Id, value);
        }
    }
}