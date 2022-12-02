using System.Collections.Generic;
using Cheetah.Matches.Realtime.Editor.DumpViewer.Sections.Objects;
using Cheetah.Matches.Realtime.Editor.DumpViewer.Sections.Objects.Query;
using Cheetah.Matches.Realtime.GRPC.Admin;
using Cheetah.Matches.Realtime.GRPC.Shared;
using Google.Protobuf;
using NUnit.Framework;

namespace Cheetah.Matches.Realtime.Tests.DumpViewer.Sections.Objects.Query
{
    public class RuleTest
    {
        [Test]
        public void ShouldFilterTemplate()
        {
            var rule = new TemplateRule(100);
            Assert.True(rule.Filter(new DumpObject()
            {
                Template = 100
            }));
            Assert.False(rule.Filter(new DumpObject()
            {
                Template = 200
            }));
        }

        [Test]
        public void ShouldFilterCreated()
        {
            var rule = new CreatedFlagRule(true);
            Assert.True(rule.Filter(new DumpObject()
            {
                Created = true
            }));
            Assert.False(rule.Filter(new DumpObject()
            {
                Created = false
            }));
        }

        [Test]
        public void ShouldFilterId()
        {
            var rule = new IdRule(100);
            Assert.True(rule.Filter(new DumpObject()
            {
                Id = 100
            }));
            Assert.False(rule.Filter(new DumpObject()
            {
                Id = 200
            }));
        }

        [Test]
        public void ShouldFilterRoomOwner()
        {
            var rule = new RoomOwnerRule();
            Assert.True(rule.Filter(new DumpObject()));
            Assert.False(rule.Filter(new DumpObject()
            {
                OwnerUserId = 100
            }));
        }

        [Test]
        public void ShouldFilterUserOwner()
        {
            var rule = new UserOwnerRule(100);
            Assert.True(rule.Filter(new DumpObject()
            {
                OwnerUserId = 100
            }));
            Assert.False(rule.Filter(new DumpObject()
            {
                OwnerUserId = 200
            }));
        }

        [Test]
        public void ShouldFilterAccessGroup()
        {
            var rule = new AccessGroupsRule(100);
            Assert.True(rule.Filter(new DumpObject()
            {
                Groups = 100
            }));
            Assert.False(rule.Filter(new DumpObject()
            {
                Groups = 200
            }));
        }

        [Test]
        public void ShouldFilterObjectByFieldId()
        {
            var rule = new FieldIdRule(100);
            Assert.True(rule.Filter(new DumpObject
            {
                Fields =
                {
                    new GameObjectField
                    {
                        Id = 100, Value = new FieldValue
                        {
                            Long = 5
                        }
                    }
                }
            }));

            Assert.True(rule.Filter(new DumpObject
            {
                Fields =
                {
                    new GameObjectField
                    {
                        Id = 100, Value = new FieldValue
                        {
                            Double = 5
                        }
                    }
                }
            }));

            Assert.True(rule.Filter(new DumpObject
            {
                Fields =
                {
                    new GameObjectField
                    {
                        Id = 100, Value = new FieldValue
                        {
                            Structure = ByteString.Empty
                        }
                    }
                }
            }));
            Assert.False(rule.Filter(new DumpObject()));
        }

        [Test]
        public void ShouldFilterFieldItemByFieldId()
        {
            var rule = new FieldIdRule(100);
            Assert.True(rule.Filter(new ObjectsViewer.FieldItem()
            {
                Id = 100
            }));
            Assert.False(rule.Filter(new ObjectsViewer.FieldItem()
            {
                Id = 200
            }));
        }

        [Test]
        public void ShouldNotRuleForObject()
        {
            var rule = new NotRule(new TemplateRule(100));
            Assert.True(rule.Filter(new DumpObject
            {
                Template = 200
            }));
            Assert.False(rule.Filter(new DumpObject
            {
                Template = 100
            }));
        }

        [Test]
        public void ShouldNotRuleForField()
        {
            var rule = new NotRule(new FieldIdRule(100));
            Assert.True(rule.Filter(new ObjectsViewer.FieldItem
            {
                Id = 200
            }));
            Assert.False(rule.Filter(new ObjectsViewer.FieldItem()
            {
                Id = 100
            }));
        }

        [Test]
        public void ShouldAndForObject()
        {
            var rule = new AndRule(new List<Rule>
            {
                new TemplateRule(100), new CreatedFlagRule(true)
            });
            Assert.True(rule.Filter(new DumpObject
            {
                Template = 100,
                Created = true
            }));
            Assert.False(rule.Filter(new DumpObject
            {
                Template = 100,
                Created = false
            }));
        }

        [Test]
        public void ShouldAndForField()
        {
            var rule = new AndRule(new List<Rule>
            {
                new FieldIdRule(100)
            });
            Assert.True(rule.Filter(new ObjectsViewer.FieldItem()
            {
                Id = 100,
            }));
            Assert.False(rule.Filter(new ObjectsViewer.FieldItem()
            {
                Id = 200,
            }));
        }

        [Test]
        public void ShouldOrForObject()
        {
            var rule = new OrRule(new List<Rule>
            {
                new TemplateRule(100), new CreatedFlagRule(true)
            });
            Assert.True(rule.Filter(new DumpObject
            {
                Template = 100,
                Created = true
            }));
            Assert.True(rule.Filter(new DumpObject
            {
                Template = 200,
                Created = true
            }));
            Assert.False(rule.Filter(new DumpObject
            {
                Template = 300,
                Created = false
            }));
        }

        [Test]
        public void ShouldOrForField()
        {
            var rule = new OrRule(new List<Rule>
            {
                new FieldIdRule(100),
                new FieldIdRule(200)
            });
            Assert.True(rule.Filter(new ObjectsViewer.FieldItem()
            {
                Id = 100,
            }));
            Assert.True(rule.Filter(new ObjectsViewer.FieldItem()
            {
                Id = 200,
            }));
            Assert.False(rule.Filter(new ObjectsViewer.FieldItem()
            {
                Id = 300,
            }));
        }
    }
}