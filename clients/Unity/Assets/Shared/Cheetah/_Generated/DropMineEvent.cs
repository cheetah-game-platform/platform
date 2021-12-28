using Cheetah.Matches.Relay.Codec;
using Cheetah.Matches.Relay.Codec.Formatter;
using Cheetah.Matches.Relay.Types;
using UnityEngine;
using Shared.Types;

// ReSharper disable once CheckNamespace
namespace Shared.Types
{
		public class DropMineEventCodec:Codec<DropMineEvent>
		{
			public void Decode(ref CheetahBuffer buffer, ref DropMineEvent dest)
			{
				dest.MineId = PrimitiveReaders.ReadInt(ref buffer);
			}
	
			public void  Encode(ref DropMineEvent source, ref CheetahBuffer buffer)
			{
				PrimitiveWriters.Write(source.MineId,ref buffer);
			}
	
	
			[RuntimeInitializeOnLoadMethod(RuntimeInitializeLoadType.SubsystemRegistration)]
			static void OnRuntimeMethodLoad()
			{
				CodecRegistryBuilder.RegisterDefault(factory=>new DropMineEventCodec());
			}
	
		}
	
	
}
