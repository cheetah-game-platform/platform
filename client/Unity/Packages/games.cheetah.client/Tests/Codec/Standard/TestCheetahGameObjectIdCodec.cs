using Games.Cheetah.Client.Codec.Standard;
using Games.Cheetah.Client.Types;
using NUnit.Framework;

namespace Games.Cheetah.Client.Tests.Codec.Standard
{
    public class TestCheetahGameObjectIdCodec
    {
        [Test]
        [TestCase(true, (ushort)0)]
        [TestCase(false, (ushort)125)]
        public void TestCodec(bool roomOwner, ushort memberId)
        {
            var id = new CheetahObjectId()
            {
                id = 100,
                roomOwner = roomOwner,
                memberId = memberId
            };
            var buffer = new CheetahBuffer();
            var codec = new CheetahObjectIdCodec();
            codec.Encode(in id, ref buffer);

            var decoded = new CheetahObjectId();
            codec.Decode(ref buffer, ref decoded);
            Assert.AreEqual(id, decoded);
        }
    }
}